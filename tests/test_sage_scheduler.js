const assert = require('node:assert/strict');
const {
  nextWave, executeWave, buildRunsIndex, isBenignStderr, visibleUserPrompt,
  shouldInspectCollabCandidate, selectStopRunIds,
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
} = require('../static/sage-scheduler.js');

const assignments = { planning: 'claude', coding: 'codex' };

let wave = nextWave(
  new Set(['planning', 'coding']),
  {},
  { planning: [], coding: [] },
  assignments
);
assert.deepEqual(wave.groups, [
  { agent: 'claude', names: ['planning'] },
  { agent: 'codex', names: ['coding'] },
]);

wave = nextWave(
  new Set(['planning', 'coding']),
  {},
  { planning: [], coding: ['planning'] },
  assignments
);
assert.deepEqual(wave.groups, [{ agent: 'claude', names: ['planning'] }]);

wave = nextWave(
  new Set(['coding']),
  { planning: { ok: true } },
  { coding: ['planning'] },
  assignments
);
assert.deepEqual(wave.groups, [{ agent: 'codex', names: ['coding'] }]);

wave = nextWave(
  new Set(['coding']),
  { planning: { ok: false } },
  { coding: ['planning'] },
  assignments
);
assert.deepEqual(wave.blocked, [{ name: 'coding', failedDependency: 'planning' }]);

wave = nextWave(
  new Set(['planning', 'coding']),
  {},
  { planning: ['coding'], coding: ['planning'] },
  assignments
);
assert.equal(wave.cycle, true);

let runs = buildRunsIndex([
  { run_id: 'run-100-0', session_id: 'same', running: false, ok: false, prompt: '真实任务' },
  { run_id: 'run-200-0', session_id: 'same', running: false, ok: true, prompt: '' },
  { run_id: 'run-150-0', session_id: 'active', running: false, ok: false },
  { run_id: 'run-140-0', session_id: 'active', running: true, ok: null },
]);
assert.equal(runs.same.ok, true);
assert.equal(runs.same.display_prompt, '真实任务');
assert.equal(runs.active.running, true);
assert.equal(isBenignStderr('2026-08-25T01:41:07Z WARN codex_core_plugins::remote sync failed'), true);
assert.equal(isBenignStderr('fatal: could not start model process'), false);
assert.equal(
  visibleUserPrompt(
    '【SAGE COLLABORATE · coding】\n任务所有者：Claude\n\n原始任务：\nCMS也要加上啊\n\n请完成本节点'
  ),
  'CMS也要加上啊'
);
assert.equal(
  visibleUserPrompt(
    '【SAGE COLLABORATE · 所有者汇总】\n任务所有者：Claude\n\n原始任务：\nCMS也要加上啊\n\n节点产出：\n...'
  ),
  ''
);
assert.equal(shouldInspectCollabCandidate('CMS也要加上啊', '代码提交推送了嘛', true), true);
assert.equal(shouldInspectCollabCandidate('CMS也要加上啊', '代码提交推送了嘛', false), false);

const stopRuns = [
  { run_id: 'run-owner', session_id: 'owner', running: true, sage: { workflow_id: 'wf-1' } },
  { run_id: 'run-child', session_id: 'child', running: true, sage: { workflow_id: 'wf-1' } },
  { run_id: 'run-finished', session_id: 'old', running: false, sage: { workflow_id: 'wf-1' } },
  { run_id: 'run-other', session_id: 'other', running: true, sage: { workflow_id: 'wf-2' } },
];
assert.deepEqual(
  selectStopRunIds(stopRuns, { localIds: ['run-owner'], sessionId: 'owner' }),
  ['run-child', 'run-owner']
);
assert.deepEqual(
  selectStopRunIds(stopRuns, { sessionId: 'child' }),
  ['run-child', 'run-owner']
);
assert.deepEqual(
  selectStopRunIds(stopRuns, { partnerSessionIds: ['child'] }),
  ['run-child', 'run-owner']
);
assert.deepEqual(
  collabLinkFromRun({
    agent: 'codex',
    session_id: 'target',
    sage: {
      kind: 'handoff',
      workflow_id: 'flow-1',
      source_agent: 'claude',
      source_session_id: 'origin',
      executor: 'Codex · gpt-5.6-sol',
    },
  }),
  {
    primaryKey: 'claude:origin',
    partnerKey: 'codex:target',
    entry: {
      partner: 'codex:target',
      executor: 'Codex · gpt-5.6-sol',
      model: null,
      effort: null,
      label: 'Codex · gpt-5.6-sol',
      kind: 'handoff',
      cats: '',
      workflow_id: 'flow-1',
      ts: 0,
    },
  }
);
assert.equal(collabLinkFromRun({ agent: 'codex', session_id: 'plain', sage: null }), null);
const handoffRun = {
  agent: 'codex', session_id: 'target', running: true,
  sage: {
    kind: 'handoff', source_agent: 'claude', source_session_id: 'origin',
    executor: 'Codex · gpt-5.6-sol',
  },
};
// HANDOFF 是所有权移交：目标会话就是接下来的主会话，必须留在侧栏
assert.equal(
  isNestedSageSession(
    { agent: 'codex', id: 'target' },
    { target: handoffRun },
    {}
  ),
  false
);
// COLLABORATE 的搭档才是子会话（只在主会话右侧面板出现）
const collabRun = {
  agent: 'codex', session_id: 'partner', running: true,
  sage: {
    kind: 'collaborate', source_agent: 'claude', source_session_id: 'owner',
    executor: 'Codex · gpt-5.6-terra', requirement: 'coding',
  },
};
assert.equal(
  isNestedSageSession({ agent: 'codex', id: 'partner' }, { partner: collabRun }, {}),
  true
);
assert.equal(
  isNestedSageSession({ agent: 'codex', id: 'plain' }, { partner: collabRun }, {}),
  false
);
// 已落盘的关联同样按 kind 区分，而不是「只要被指向就算子会话」
const nestedStore = {
  links: {
    'claude:owner': [
      { partner: 'codex:partner', kind: 'pipeline' },
      { partner: 'codex:taken-over', kind: 'handoff' },
    ],
  },
  back: { 'codex:partner': 'claude:owner', 'codex:taken-over': 'claude:owner' },
};
assert.equal(isNestedSageSession({ agent: 'codex', id: 'partner' }, {}, nestedStore), true);
assert.equal(isNestedSageSession({ agent: 'codex', id: 'taken-over' }, {}, nestedStore), false);
const legacySessions = [
  { agent: 'claude', id: 'older', updated: '2026-08-25T02:50:00Z' },
  { agent: 'claude', id: 'source', updated: '2026-08-25T03:03:00Z' },
  { agent: 'codex', id: 'target', created: '2026-08-25T03:09:00Z' },
];
assert.equal(
  isLegacyHandoffSource(legacySessions[1], legacySessions[2], legacySessions),
  true
);
assert.equal(
  isLegacyHandoffSource(legacySessions[0], legacySessions[2], legacySessions),
  false
);
assert.equal(
  isLegacyHandoffSource(
    legacySessions[1],
    { agent: 'codex', id: 'late', created: '2026-08-25T03:30:00Z' },
    legacySessions
  ),
  false
);
const legacyTarget = {
  ...legacySessions[2],
  sage: { kind: 'handoff', executor: 'Codex · sol', workflow_id: 'legacy-flow' },
};
const legacyLinks = collabLinksFromSessions([
  legacySessions[0], legacySessions[1], legacyTarget,
]);
assert.equal(legacyLinks.length, 1);
assert.equal(legacyLinks[0].primaryKey, 'claude:source');
assert.equal(legacyLinks[0].partnerKey, 'codex:target');
const workflowLinks = collabLinksFromSessions([
  {
    agent: 'codex', id: 'workflow-owner',
    sage: {
      kind: 'collaborate', workflow_id: 'workflow-1',
      owner: 'Codex · sol', executor: 'Codex · sol', requirement: 'debugging',
    },
  },
  {
    agent: 'claude', id: 'workflow-child',
    sage: {
      kind: 'collaborate', workflow_id: 'workflow-1',
      owner: 'Codex · sol', executor: 'Claude Code · fable', requirement: 'vision',
    },
  },
]);
assert.equal(workflowLinks.length, 1);
assert.equal(workflowLinks[0].primaryKey, 'codex:workflow-owner');
assert.equal(workflowLinks[0].partnerKey, 'claude:workflow-child');
assert.equal(shouldReloadInstance(null, 'instance-a'), false);
assert.equal(shouldReloadInstance('instance-a', 'instance-a'), false);
assert.equal(shouldReloadInstance('instance-a', 'instance-b'), true);
assert.equal(
  collabLinkFromRun({
    agent: 'claude',
    session_id: 'partner',
    sage: {
      kind: 'collaborate',
      workflow_id: 'flow-2',
      source_agent: 'codex',
      source_session_id: 'owner',
      executor: 'Claude Code · fable',
      requirement: 'analysis',
    },
  }).entry.kind,
  'pipeline'
);

(async () => {
  const trace = [];
  const activeAgents = new Set();
  let activeTotal = 0;
  let maxActive = 0;
  await executeWave(
    [
      { agent: 'codex-a', names: ['debugging', 'review'] },
      { agent: 'terra', names: ['coding'] },
    ],
    async (agent, name) => {
      assert.equal(activeAgents.has(agent), false, `同 agent 不得并行：${agent}`);
      activeAgents.add(agent);
      activeTotal += 1;
      maxActive = Math.max(maxActive, activeTotal);
      trace.push(`start:${name}`);
      await new Promise((resolve) => setTimeout(resolve, name === 'debugging' ? 20 : 5));
      trace.push(`end:${name}`);
      activeTotal -= 1;
      activeAgents.delete(agent);
    }
  );
  assert.ok(maxActive >= 2, '不同 agent 的独立节点应当并行');
  assert.ok(trace.indexOf('start:coding') < trace.indexOf('end:debugging'));
  assert.ok(trace.indexOf('start:review') > trace.indexOf('end:debugging'));
  console.log('sage scheduler tests passed');
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

// 跨会话上下文转移：来源会话记录附在任务之后，气泡里只显示原始任务
const HANDOFF_CTX =
  '【SAGE HANDOFF】路由判定你接管本任务的完整所有权。\n' +
  '协作标识：flow-1\n来源会话：codex:origin\n\n' +
  '生成文档\n\n【来源会话上下文】你接手的是一个已在进行中的对话。\n\n〔用户〕检查对齐\n\n〔Codex〕结论如下';
assert.equal(visibleUserPrompt(HANDOFF_CTX), '生成文档');
const COLLAB_CTX =
  '【SAGE COLLABORATE · coding】\n任务所有者：Claude\n当前执行者：Codex\n\n' +
  '原始任务：\n所以最终结果是什么？\n\n【来源会话上下文】〔用户〕之前的问题\n\n' +
  '请完成本节点并给出可供下游节点直接使用的明确产出。';
assert.equal(visibleUserPrompt(COLLAB_CTX), '所以最终结果是什么？');

console.log('sage context-transfer tests passed');

// 追问再次被判定为移交时，复用同一条分支而不是每次新建会话
const handoffLinks = [
  { partner: 'claude:newest', kind: 'handoff', executor: 'claude::sonnet', title: '生成文档', ts: 300 },
  { partner: 'codex:helper', kind: 'pipeline', executor: 'codex::terra', ts: 400 },
  { partner: 'claude:older', kind: 'handoff', executor: 'Claude Code · sonnet · low', ts: 200 },
];
// 数组顺序不代表新旧，必须按时间戳取最近一条
assert.equal(reusableHandoffLink(handoffLinks, 'claude').partner, 'claude:newest');
// executor 字段在两个写入点分别存过 id 和 label，不能作为匹配键
assert.equal(reusableHandoffLink(handoffLinks, 'codex'), null);
// 同一 CLI 换模型仍是同一条分支，不再劈新会话
assert.equal(
  reusableHandoffLink([{ partner: 'codex:legacy', kind: 'handoff' }], 'codex').partner,
  'codex:legacy'
);
// 时间戳都缺失时以数组靠后者为准
assert.equal(
  reusableHandoffLink(
    [{ partner: 'claude:a', kind: 'handoff' }, { partner: 'claude:b', kind: 'handoff' }],
    'claude'
  ).partner,
  'claude:b'
);
assert.equal(reusableHandoffLink([], 'claude'), null);
assert.equal(reusableHandoffLink(null, 'claude'), null);

console.log('sage handoff-reuse tests passed');

// ---- 同 runtime 的移交原地接管，不另开会话 ----
const inPlaceDecision = { mode: 'handoff', primary: 'codex::gpt-5.2' };
assert.equal(
  handoffStaysInPlace(inPlaceDecision, { agent: 'codex', id: 'live' }, 'codex'),
  true
);
// 跨 CLI 的移交仍必须新建会话（两家会话文件互不通用）
assert.equal(
  handoffStaysInPlace(inPlaceDecision, { agent: 'codex', id: 'live' }, 'claude'),
  false
);
// 还没有会话（Hero 首轮）时无所谓「原地」
assert.equal(handoffStaysInPlace(inPlaceDecision, { agent: 'codex', id: null }, 'codex'), false);
assert.equal(handoffStaysInPlace(inPlaceDecision, null, 'codex'), false);
// self / collaborate 不受影响
assert.equal(
  handoffStaysInPlace({ mode: 'self', primary: 'codex::gpt-5.2' }, { agent: 'codex', id: 'live' }, 'codex'),
  false
);

console.log('sage in-place handoff tests passed');

// ---- 侧栏分组：移交子会话挂到来源会话下 ----
const treeSessions = [
  {
    agent: 'codex', id: 'child', updated: '2026-08-25T08:31:39Z',
    sage: { kind: 'handoff', source_agent: 'codex', source_session_id: 'parent' },
  },
  { agent: 'codex', id: 'parent', updated: '2026-08-25T08:30:31Z' },
  { agent: 'claude', id: 'solo', updated: '2026-08-25T08:40:00Z' },
];
const tree = buildSessionTree(treeSessions, {}, {});
assert.deepEqual(
  tree.map((node) => [node.session.agent + ':' + node.session.id, node.children.length]),
  [['claude:solo', 0], ['codex:parent', 1]]
);
// 分支排序看子会话的时间：父会话没更新也不该沉底
assert.equal(tree[1].children[0].id, 'child');
// 来源会话不在本次列表里（被 limit 截断 / 跨项目）→ 子会话回退成顶层条目
const orphan = buildSessionTree([treeSessions[0]], {}, {});
assert.deepEqual(orphan.map((node) => node.session.id), ['child']);
// 连续移交 A→B→C 折叠到根，只缩进一层
const chain = buildSessionTree(
  [
    { agent: 'codex', id: 'a', updated: '2026-08-25T01:00:00Z' },
    {
      agent: 'codex', id: 'b', updated: '2026-08-25T02:00:00Z',
      sage: { kind: 'handoff', source_agent: 'codex', source_session_id: 'a' },
    },
    {
      agent: 'claude', id: 'c', updated: '2026-08-25T03:00:00Z',
      sage: { kind: 'handoff', source_agent: 'codex', source_session_id: 'b' },
    },
  ],
  {}, {}
);
assert.equal(chain.length, 1);
assert.equal(chain[0].session.id, 'a');
assert.deepEqual(chain[0].children.map((s) => s.id), ['c', 'b']);
// 协作搭档不是移交，不做嵌套（它们本来就被 filterSessions 挡在侧栏外）
assert.equal(
  buildSessionTree(
    [
      { agent: 'claude', id: 'owner', updated: '2026-08-25T01:00:00Z' },
      {
        agent: 'codex', id: 'mate', updated: '2026-08-25T02:00:00Z',
        sage: { kind: 'collaborate', source_agent: 'claude', source_session_id: 'owner' },
      },
    ],
    {}, {}
  ).length,
  2
);
// 本地关联表里的移交同样成立；数据成环时不死循环、每条会话只出现一次
const storeTree = buildSessionTree(
  [
    { agent: 'claude', id: 'owner', updated: '2026-08-25T01:00:00Z' },
    { agent: 'codex', id: 'taken-over', updated: '2026-08-25T02:00:00Z' },
  ],
  {},
  {
    links: { 'claude:owner': [{ partner: 'codex:taken-over', kind: 'handoff' }] },
    back: { 'codex:taken-over': 'claude:owner' },
  }
);
assert.deepEqual(
  storeTree.map((node) => [node.session.id, node.children.map((s) => s.id)]),
  [['owner', ['taken-over']]]
);
const cyclic = buildSessionTree(
  [
    { agent: 'codex', id: 'x', sage: { kind: 'handoff', source_agent: 'codex', source_session_id: 'y' } },
    { agent: 'codex', id: 'y', sage: { kind: 'handoff', source_agent: 'codex', source_session_id: 'x' } },
  ],
  {}, {}
);
assert.equal(cyclic.length + cyclic.reduce((sum, node) => sum + node.children.length, 0), 2);
// 运行注册表兜底：会话文件还没落盘 sage 元数据时也能配对
assert.equal(
  handoffParentKey(
    { agent: 'codex', id: 'target' },
    { target: handoffRun },
    {}
  ),
  'claude:origin'
);
assert.equal(handoffParentKey({ agent: 'codex', id: 'plain' }, {}, {}), null);

console.log('sage session-tree tests passed');

// ---- 一个会话只路由一次：首轮定执行者，之后沿用 ----
const SESS = { agent: 'codex', id: 'live-1' };
const base = { sageOn: true, session: SESS, text: '继续看看这个', hasAttachments: false };
// 首轮：还没被决策过 → 路由
assert.equal(shouldRouteFollowUp({ ...base, decided: false, retrying: false }), true);
// 之后的追问：已定执行者 → 不再重新决策（否则每轮被学习噪声带着换模型）
assert.equal(shouldRouteFollowUp({ ...base, decided: true, retrying: false }), false);
// 例外一：上一轮执行失败 → 重路由，好把失败的执行者换掉
assert.equal(shouldRouteFollowUp({ ...base, decided: true, retrying: true }), true);
// 开关关掉就完全不路由
assert.equal(shouldRouteFollowUp({ ...base, sageOn: false, decided: false }), false);
// 斜杠命令走 CLI 内置命令，不参与路由
assert.equal(shouldRouteFollowUp({ ...base, text: '/review', decided: false }), false);
// 带图/附件的消息不路由（关键词推断对附件无能为力）
assert.equal(shouldRouteFollowUp({ ...base, hasAttachments: true, decided: false }), false);
// 空文本、无会话（Hero 首轮走另一条分支）
assert.equal(shouldRouteFollowUp({ ...base, text: '', decided: false }), false);
assert.equal(shouldRouteFollowUp({ ...base, session: null, decided: false }), false);
assert.equal(shouldRouteFollowUp({ ...base, session: { agent: 'codex', id: null }, decided: false }), false);
assert.equal(shouldRouteFollowUp(), false);

console.log('sage route-once tests passed');
