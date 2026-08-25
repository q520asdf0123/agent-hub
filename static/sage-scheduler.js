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
      },
    };
  }

  /** 只有 COLLABORATE 的搭档算「子会话」（藏进主会话右侧面板）。
   *  HANDOFF 是所有权移交，目标会话就是接下来的主会话，必须留在侧栏；
   *  否则智能路由每移交一次，侧栏就少一条记录。 */
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
      seen.add(link.partnerKey);
      links.push(link);
    }
    return links;
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
    isLegacyHandoffSource,
    collabLinksFromSessions,
    shouldReloadInstance,
  };
})();

if (typeof module === 'object' && module.exports) module.exports = SageScheduler;
if (typeof document === 'object') document.documentElement.dataset.sageScheduler = 'ready';
