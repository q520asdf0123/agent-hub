var SageScheduler = (function () {
  /** 跨会话上下文转移块的起始标记，与 src/history/claude.rs 保持一致 */
  const SAGE_CONTEXT_MARKER = '\n\n【来源会话上下文】';

  function nextWave(pendingNames, results, dependencies, assignments) {
    const pending = [...pendingNames];
    const ready = pending.filter((name) =>
      (dependencies[name] || []).every((dependency) =>
        Object.prototype.hasOwnProperty.call(results, dependency)
      )
    );
    if (!ready.length) return { cycle: pending.length > 0, blocked: [], groups: [] };

    const blocked = [];
    const grouped = new Map();
    for (const name of ready) {
      const failedDependency = (dependencies[name] || []).find(
        (dependency) => !results[dependency].ok
      );
      if (failedDependency) {
        blocked.push({ name, failedDependency });
        continue;
      }
      const agent = assignments[name];
      if (!grouped.has(agent)) grouped.set(agent, []);
      grouped.get(agent).push(name);
    }
    const groups = [...grouped.entries()]
      .sort(([left], [right]) => String(left).localeCompare(String(right)))
      .map(([agent, names]) => ({ agent, names: names.sort() }));
    return { cycle: false, blocked, groups };
  }

  /** SAGE 官方调度语义：同 agent 的 names 串行；不同 agent groups 并行。 */
  async function executeWave(groups, runRequirement) {
    await Promise.all(
      (groups || []).map(async ({ agent, names }) => {
        for (const name of names || []) {
          await runRequirement(agent, name);
        }
      })
    );
  }

  function buildRunsIndex(runs) {
    const index = {};
    const visiblePrompts = {};
    const timestamp = (run) =>
      parseInt(String((run && run.run_id) || '').split('-')[1] || '0', 10);
    for (const run of runs || []) {
      if (!run.session_id) continue;
      if (String(run.prompt || '').trim()) {
        const previousPrompt = visiblePrompts[run.session_id];
        if (!previousPrompt || timestamp(run) > previousPrompt.timestamp) {
          visiblePrompts[run.session_id] = {
            timestamp: timestamp(run),
            prompt: String(run.prompt).trim(),
          };
        }
      }
      const previous = index[run.session_id];
      if (
        !previous ||
        (run.running && !previous.running) ||
        (!!run.running === !!previous.running && timestamp(run) > timestamp(previous))
      ) {
        index[run.session_id] = run;
      }
    }
    for (const [sessionId, run] of Object.entries(index)) {
      run.display_prompt = visiblePrompts[sessionId]?.prompt || String(run.prompt || '').trim();
    }
    return index;
  }

  function isBenignStderr(text) {
    const value = String(text || '');
    if (!value.trim()) return true;
    return (
      /(^|\s)WARN(\s|$)/.test(value) ||
      value.includes('Skill descriptions were shortened to fit the skills context budget') ||
      value.includes('will be omitted') ||
      value.includes('dangerously-bypass-hook-trust')
    );
  }

  function visibleUserPrompt(text) {
    const value = String(text || '').trim();
    if (!value) return '';
    if (value.startsWith('【SAGE HANDOFF】')) {
      const split = value.indexOf('\n\n');
      if (split < 0) return '';
      const rest = value.slice(split + 2);
      const end = rest.indexOf(SAGE_CONTEXT_MARKER);
      return (end >= 0 ? rest.slice(0, end) : rest).trim();
    }
    if (value.startsWith('【SAGE COLLABORATE')) {
      if (value.startsWith('【SAGE COLLABORATE · 所有者汇总】')) return '';
      const marker = '原始任务：';
      const start = value.indexOf(marker);
      if (start < 0) return '';
      const rest = value.slice(start + marker.length).replace(/^[\r\n]+/, '');
      const ends = [SAGE_CONTEXT_MARKER, '\n\n依赖节点产出：', '\n\n节点产出：', '\n\n请完成本节点']
        .map((item) => rest.indexOf(item))
        .filter((index) => index >= 0);
      return rest.slice(0, ends.length ? Math.min(...ends) : rest.length).trim();
    }
    if (
      ['【协作分工】', '【协作汇总】', '【协作复查回注】', '【协作复查】', '【协作追问】']
        .some((prefix) => value.startsWith(prefix)) ||
      value.startsWith('<task-notification>') ||
      value.startsWith('This session is being continued from a previous conversation')
    ) {
      return '';
    }
    return value;
  }

  function shouldInspectCollabCandidate(sessionTitle, taskTitle, hasWorkflowId) {
    if (hasWorkflowId) return true;
    if (!taskTitle) return true;
    return String(sessionTitle || '').includes(String(taskTitle));
  }

  function selectStopRunIds(runs, options) {
    const settings = options || {};
    const localIds = new Set(settings.localIds || []);
    const sessionIds = new Set([
      settings.sessionId,
      ...(settings.partnerSessionIds || []),
    ].filter(Boolean));
    const running = (runs || []).filter((run) => run && run.running);
    const seeds = running.filter(
      (run) => localIds.has(run.run_id) || sessionIds.has(run.session_id)
    );
    const workflowIds = new Set([
      ...(settings.workflowIds || []),
      ...seeds.map((run) => run.sage && run.sage.workflow_id),
    ].filter(Boolean));
    const ids = new Set(localIds);
    for (const run of running) {
      if (
        localIds.has(run.run_id) ||
        sessionIds.has(run.session_id) ||
        (run.sage && workflowIds.has(run.sage.workflow_id))
      ) {
        ids.add(run.run_id);
      }
    }
    return [...ids].sort();
  }

  function collabLinkFromRun(run) {
    const meta = run && run.sage;
    if (
      !run || !run.agent || !run.session_id || !meta ||
      !meta.source_agent || !meta.source_session_id ||
      !['collaborate', 'handoff'].includes(meta.kind)
    ) {
      return null;
    }
    const partnerKey = `${run.agent}:${run.session_id}`;
    const primaryKey = `${meta.source_agent}:${meta.source_session_id}`;
    if (partnerKey === primaryKey) return null;
    return {
      primaryKey,
      partnerKey,
      entry: {
        partner: partnerKey,
        executor: meta.executor || run.agent,
        model: null,
        effort: null,
        label: meta.executor || run.agent,
        kind: meta.kind === 'handoff' ? 'handoff' : 'pipeline',
        cats: meta.requirement || '',
        workflow_id: meta.workflow_id || null,
        // 复用移交分支要挑最近一条，没有时间戳就只能靠数组顺序，不可靠
        ts: parseInt(String(run.run_id || '').split('-')[1] || '0', 10) || 0,
      },
    };
  }

  /** 只有 COLLABORATE 的搭档算「子会话」（藏进主会话右侧面板）。
   *  HANDOFF 是所有权移交，目标会话就是接下来的主会话，必须留在侧栏；
   *  否则智能路由每移交一次，侧栏就少一条记录。
   *  （留在侧栏但不平铺：buildSessionTree 会把它缩进挂到来源会话下。） */
  function isNestedSageSession(session, runsIndex, store) {
    if (!session || !session.id || !session.agent) return false;
    const key = `${session.agent}:${session.id}`;
    const primaryKey = store && store.back && store.back[key];
    if (primaryKey) {
      const entry = ((store.links || {})[primaryKey] || []).find(
        (item) => item && item.partner === key
      );
      if (entry) return entry.kind !== 'handoff';
    }
    return Object.values(runsIndex || {}).some((run) => {
      const link = collabLinkFromRun(run);
      return link && link.partnerKey === key && link.entry.kind !== 'handoff';
    });
  }

  /** 同一 CLI runtime 的移交在原会话里换模型接管即可，不必另开会话：
   *  上下文本来就在这条会话里，另开只会把侧栏拆成两条、还要重灌一遍历史。
   *  跨 runtime（codex ↔ claude）仍必须新建，两家的会话文件互不通用。 */
  function handoffStaysInPlace(decision, session, executorRuntime) {
    return !!(
      decision && decision.mode === 'handoff' &&
      session && session.id && session.agent &&
      executorRuntime && executorRuntime === session.agent
    );
  }

  /** 追问要不要重新路由。一个会话只路由一次：首轮定执行者，之后一律沿用。
   *
   *  上游 SAGE 是「给定一个 Task 选最优配置」的路由器，一个任务路由一次。把每轮追问
   *  都当新 Task 反复决策，会被本机学习状态里的小样本噪声左右——实测同一批提问，
   *  空学习状态 5/5 留任，带上仅 8 次更新的真实状态后 0/5，每轮换个模型，
   *  接手的模型还得重新理解一遍上下文。
   *
   *  两个例外仍要重新路由：上一轮执行失败（官方失败重路由语义），
   *  以及这条会话还没被决策过（终端建的、刚打开开关的——那就是它的首轮）。 */
  function shouldRouteFollowUp(options) {
    const o = options || {};
    if (!o.sageOn || !o.session || !o.session.id) return false;
    if (!o.text || String(o.text).startsWith('/')) return false;
    if (o.hasAttachments) return false;
    return !o.decided || !!o.retrying;
  }

  function sessionKey(session) {
    return session && session.agent && session.id ? `${session.agent}:${session.id}` : '';
  }

  function sessionTime(session) {
    return Date.parse((session && (session.updated || session.created)) || '') || 0;
  }

  /** 会话的移交来源（父会话）key，三处依次兜底：
   *  会话自带的 sage 元数据 → 本地关联表（含回溯补配的旧记录）→ 运行注册表。 */
  function handoffParentKey(session, runsIndex, store) {
    const key = sessionKey(session);
    if (!key) return null;
    const meta = session.sage;
    if (meta && meta.kind === 'handoff' && meta.source_agent && meta.source_session_id) {
      const parent = `${meta.source_agent}:${meta.source_session_id}`;
      if (parent !== key) return parent;
    }
    const primaryKey = store && store.back && store.back[key];
    if (primaryKey && primaryKey !== key) {
      const entry = ((store.links || {})[primaryKey] || []).find(
        (item) => item && item.partner === key
      );
      if (entry && entry.kind === 'handoff') return primaryKey;
    }
    for (const run of Object.values(runsIndex || {})) {
      const link = collabLinkFromRun(run);
      if (link && link.partnerKey === key && link.entry.kind === 'handoff') return link.primaryKey;
    }
    return null;
  }

  /** 侧栏分组：移交子会话缩进挂到来源会话下，不再各占一行。
   *  连续移交 A→B→C 一律折叠到根 A（侧栏太窄，多级缩进读不清）；
   *  来源会话不在本次列表里（被 limit 截断、跨项目）的子会话回退成顶层条目。
   *  顶层按「自身与子会话中最新的时间」排序，父会话不会因为自己没更新而沉底。 */
  function buildSessionTree(sessions, runsIndex, store) {
    const list = (sessions || []).filter((session) => sessionKey(session));
    const known = new Set(list.map(sessionKey));
    const parents = new Map();
    for (const session of list) {
      const parent = handoffParentKey(session, runsIndex, store);
      if (parent && known.has(parent)) parents.set(sessionKey(session), parent);
    }
    const cache = new Map();
    const rootOf = (key) => {
      if (cache.has(key)) return cache.get(key);
      const chain = [key];
      const seen = new Set(chain);
      let cursor = key;
      while (parents.has(cursor)) {
        const next = parents.get(cursor);
        if (seen.has(next)) break; // 数据成环：就近截断，保证每条会话只出现一次
        seen.add(next);
        chain.push(next);
        cursor = next;
      }
      for (const item of chain) cache.set(item, cursor);
      return cursor;
    };
    const nodes = new Map();
    const order = [];
    for (const session of list) {
      const key = sessionKey(session);
      if (rootOf(key) !== key) continue;
      nodes.set(key, { session, children: [] });
      order.push(key);
    }
    for (const session of list) {
      const key = sessionKey(session);
      const root = rootOf(key);
      if (root !== key) nodes.get(root).children.push(session);
    }
    const tree = order.map((key) => nodes.get(key));
    for (const node of tree) {
      node.children.sort((left, right) => sessionTime(right) - sessionTime(left));
    }
    const branchTime = (node) =>
      Math.max(sessionTime(node.session), ...node.children.map(sessionTime), 0);
    return tree.sort((left, right) => branchTime(right) - branchTime(left));
  }

  function isLegacyHandoffSource(source, target, sessions) {
    if (!source || !target || !source.id || !target.id || source.agent === target.agent) return false;
    const targetTime = Date.parse(target.created || target.updated || '');
    if (!Number.isFinite(targetTime)) return false;
    const sourceTime = (session) => {
      const updated = Date.parse(session.updated || '');
      if (Number.isFinite(updated) && updated <= targetTime) return updated;
      return Date.parse(session.created || '');
    };
    const ownTime = sourceTime(source);
    if (!Number.isFinite(ownTime) || targetTime < ownTime || targetTime - ownTime > 15 * 60_000) {
      return false;
    }
    const nearest = (sessions || [])
      .filter((session) => session.agent === source.agent && session.id !== target.id)
      .map((session) => ({ session, time: sourceTime(session) }))
      .filter(({ time }) => Number.isFinite(time) && time <= targetTime && targetTime - time <= 15 * 60_000)
      .sort((left, right) => right.time - left.time)[0];
    return !!nearest && nearest.session.id === source.id;
  }

  function collabLinksFromSessions(sessions) {
    const links = [];
    const seen = new Set();
    for (const target of sessions || []) {
      let meta = target && target.sage;
      if (!target || !target.id || !target.agent || !meta) continue;
      if (meta.kind === 'collaborate' && !meta.source_session_id && meta.workflow_id) {
        const ownerSession = (sessions || []).find(
          (candidate) =>
            candidate && candidate.sage &&
            candidate.sage.kind === 'collaborate' &&
            candidate.sage.workflow_id === meta.workflow_id &&
            candidate.sage.owner && candidate.sage.executor === candidate.sage.owner
        );
        if (ownerSession) {
          meta = {
            ...meta,
            source_agent: ownerSession.agent,
            source_session_id: ownerSession.id,
          };
        }
      }
      if (meta.kind === 'handoff' && !meta.source_session_id) {
        const source = (sessions || []).find((candidate) =>
          isLegacyHandoffSource(candidate, target, sessions)
        );
        if (!source) continue;
        meta = {
          ...meta,
          source_agent: source.agent,
          source_session_id: source.id,
        };
      }
      const link = collabLinkFromRun({
        agent: target.agent,
        session_id: target.id,
        sage: meta,
      });
      if (!link || seen.has(link.partnerKey)) continue;
      link.entry.ts = Date.parse(target.created || target.updated || '') || link.entry.ts || 0;
      if (target.title) link.entry.title = target.title; // 复用分支时抬头沿用它自己的标题
      seen.add(link.partnerKey);
      links.push(link);
    }
    return links;
  }

  /** 同一来源会话已经移交给同一 runtime 时，取回那条分支（取最近一条）。
   *  每次追问都新建会话会让侧栏堆满同名条目，上下文也无法在分支里累积。
   *
   *  只按 runtime 匹配，不比 model / effort：移交的语义是「交给另一个 CLI 接管」，
   *  同一 CLI 换个模型仍是同一条分支，再劈一条只会重新制造碎片。
   *  （COLLABORATE 不能这样——同一 runtime 的多个搭档节点必须各用各的会话。）
   *  link.executor 在不同写入点分别存过 id 和 label，因此不作为匹配键。 */
  function reusableHandoffLink(links, runtime) {
    const prefix = String(runtime || '') + ':';
    let best = null;
    for (const link of links || []) {
      if (!link || link.kind !== 'handoff') continue;
      if (!String(link.partner || '').startsWith(prefix)) continue;
      // 取时间戳最大的一条；时间戳缺失（旧记录）时以数组靠后者为准
      if (!best || (Number(link.ts) || 0) >= (Number(best.ts) || 0)) best = link;
    }
    return best;
  }

  function shouldReloadInstance(currentId, nextId) {
    return !!currentId && !!nextId && currentId !== nextId;
  }

  return {
    SAGE_CONTEXT_MARKER,
    nextWave,
    executeWave,
    buildRunsIndex,
    isBenignStderr,
    visibleUserPrompt,
    shouldInspectCollabCandidate,
    selectStopRunIds,
    collabLinkFromRun,
    isNestedSageSession,
    handoffStaysInPlace,
    shouldRouteFollowUp,
    handoffParentKey,
    buildSessionTree,
    isLegacyHandoffSource,
    collabLinksFromSessions,
    reusableHandoffLink,
    shouldReloadInstance,
  };
})();

if (typeof module === 'object' && module.exports) module.exports = SageScheduler;
if (typeof document === 'object') document.documentElement.dataset.sageScheduler = 'ready';
