const assert = require('node:assert/strict');
const {
  nextWave, executeWave, buildRunsIndex, isBenignStderr, visibleUserPrompt,
  shouldInspectCollabCandidate, selectStopRunIds,
  collabLinkFromRun,
  isNestedSageSession,
  isLegacyHandoffSource,
  collabLinksFromSessions,
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
