'use strict';

/* ==========================================================================
 * i18n：中文为源语言（键即中文），英文经 I18N_EN 映射；t() 全局可用。
 * ========================================================================== */

let CUR_LANG = 'zh';

const I18N_EN = {
  '搜索会话': 'Search sessions', '新建会话': 'New session', '项目': 'Projects', '对话': 'Chats',
  '在文本框中显示': 'Show in input', '移除': 'Remove', '点击展开 / 收起': 'Click to expand / collapse',
  '复查': 'review', '🤝 追问分派任务书': '🤝 Delegated follow-up brief',
  '复制正文': 'Copy text', '已复制': 'Copied', '复制失败': 'Copy failed',
  '分支到新会话：基于当前会话状态分叉，原会话不受影响': 'Fork to a new session from the current state (original untouched)',
  '会话尚未保存，无法分支': 'Session not saved yet — cannot fork',
  '🗑 删除会话': '🗑 Delete session', '已删除': 'Deleted', '删除失败：': 'Delete failed: ',
  '分支失败：': 'Fork failed: ',
  '从此消息处分叉出新会话（之后的内容不带入，原会话不受影响）': 'Fork a new session from this message (later content excluded; original untouched)',
  '会话有任务在运行，先停止再删除': 'A task is running in this session — stop it first',
  '确定删除该会话？文件将移入回收目录（可恢复）': 'Delete this session? The file moves to a recoverable trash folder.',
  '🧠 记忆': '🧠 Memory',
  '👥 团队': '👥 Team',
  '🤝 协作分工任务书': '🤝 Work-division brief', '🤝 分工产出回注': '🤝 Consolidated partner output',
  '🤝 复查意见回注': '🤝 Review feedback', '🤝 协作复查任务书': '🤝 Review brief',
  '搜索结果': 'Results', '技能': 'Skills', '图片': 'Image', '本地': 'Local', '收起': 'Collapse',
  '收起文件 ⌃': 'Collapse files ⌃', '已处理': 'Processed in', '(无标题)': '(untitled)',
  '暂无对话': 'No chats yet', '没有匹配的会话': 'No matching sessions', '搜索失败：': 'Search failed: ',
  '会话加载失败': 'Failed to load sessions', '项目加载失败': 'Failed to load projects',
  '还未导入项目，点「＋」从历史中选择': 'No projects imported — click "+" to pick from history',
  '检测 CLI 中…': 'Detecting CLIs…', '已安装': 'installed', '未安装': 'not installed',
  'CLI 状态获取失败': 'Failed to get CLI status', '切换 Agent': 'Switch agent',
  '默认模型': 'Default model', '默认': 'Default', '绕过权限': 'Bypass permissions',
  '接受编辑': 'Accept edits', '计划': 'Plan', '工作区可写': 'Workspace write', '只读': 'Read-only',
  '低': 'Low', '中': 'Medium', '高': 'High', '超高': 'X-High', '最大': 'Max', '最小': 'Minimal',
  '全局默认': 'global default', '加载技能中…': 'Loading skills…',
  '筛选技能 / 命令…': 'Filter skills / commands…', '没有匹配的技能': 'No matching skills',
  '未发现技能': 'No skills found',
  '未发现 Codex 自定义 prompt（~/.codex/prompts/*.md）': 'No Codex custom prompts (~/.codex/prompts/*.md)',
  '技能加载失败：': 'Failed to load skills: ',
  '搜索历史项目（名称 / 路径）…': 'Search historical projects…', '扫描历史项目中…': 'Scanning history…',
  '没有匹配的项目': 'No matching projects', '历史中没有发现项目': 'No projects found in history',
  '扫描失败：': 'Scan failed: ', '⌨ 手动输入路径…': '⌨ Enter a path manually…',
  '导入': 'Import', '移除': 'Remove', '会话': 'sessions', '操作失败：': 'Operation failed: ',
  '添加项目失败：': 'Failed to add project: ',
  '输入项目目录的完整路径': 'Enter the full project directory path', '输入模型名': 'Enter model name',
  '输入你的任务…': 'Describe your task…', '继续这个会话…': 'Continue this session…',
  '请先选择项目目录（输入卡片下方的项目选择器）': 'Pick a project directory first (selector below the composer)',
  '图片上传失败：': 'Image upload failed: ', '请求失败：': 'Request failed: ',
  '运行失败（无错误信息）': 'Run failed (no error message)', '■ 已停止': '■ Stopped',
  '↪ 已断开查看，任务在后台继续': '↪ Viewer detached — task continues in background',
  '页面重连 · 以下为实时输出': 'Reconnected · live output below', '加载差异中…': 'Loading diff…',
  '没有可显示的差异（可能已提交，或与 HEAD 一致）': 'No diff to show (maybe committed, or identical to HEAD)',
  '差异加载失败：': 'Failed to load diff: ', '✕ 关闭': '✕ Close',
  '在 VS Code 中打开': 'Open in VS Code', '在资源管理器中打开': 'Reveal in File Explorer',
  '在终端中打开': 'Open in Terminal', '用资源管理器打开': 'Open in File Explorer',
  '复制路径': 'Copy path', '复制目录路径': 'Copy folder path', '复制文件内容': 'Copy file content', '复制失败：': 'Copy failed: ',
  '打开失败：': 'Open failed: ', '⧉ 复制': '⧉ Copy', '✓ 已复制': '✓ Copied', '▶ 预览': '▶ Preview',
  '✕ 收起预览': '✕ Hide preview', '⧉ 新标签打开': '⧉ Open in new tab', '💭 思考过程': '💭 Thinking',
  '子 agent 过程': 'Sub-agent activity',
  '📋 任务计划': '📋 Plan', '运行中': 'Running', '已完成': 'Done', '运行出错：': 'Failed: ',
  '未知错误': 'unknown error', '转录加载失败：': 'Failed to load transcript: ', '重试': 'Retry',
  '（此会话没有可显示的消息）': '(no displayable messages in this session)',
  '编辑了文件': 'Edited files', '读取了文件': 'Read files', '运行了命令': 'Ran commands',
  '执行了操作': 'Performed actions', '⚡ 快速': '⚡ Fast', '⚡ 快速·开': '⚡ Fast · on',
  '🧭 智能路由': '🧭 Smart routing', '🧭 智能路由·开': '🧭 Routing · on', '🧭 路由中…': '🧭 Routing…',
  '🧭 已定执行者': '🧭 Executor locked',
  '🧭 SAGE 路由': '🧭 SAGE routing', '继续当前': 'Stay', '移交': 'Handoff', '协作': 'Collaborate',
  '需求推断：': 'Inferred needs: ', '协作：完成后由 ': 'Collaborate: reviewed by ',
  ' 复查，结论回注收尾': ', findings fed back to wrap up',
  '执行顺序：': 'Order: ', ' 先做自己的部分 → 搭档 ': ' does its part first → partner ',
  ' 接力 → 回注汇总': ' takes over → consolidate back',
  '成功率 ': 'Success ', ' · 覆盖 ': ' · Coverage ', ' · 效用 ': ' · Utility ',
  '点击查看差异 · 右键更多操作': 'Click for diff · right-click for more', '缓存': 'Cache', '上下文': 'Context', '点击放大': 'Click to zoom',
  '刚刚': 'just now', '跟随浏览器': 'Follow browser', '选择项目…': 'Pick a project…',
  '移除图片': 'Remove image', '＋ 导入项目…': '+ Import project…',
  'Enter 发送 · Shift+Enter 换行': 'Enter to send · Shift+Enter for newline',
};

function t(s) {
  if (CUR_LANG !== 'en') return s;
  const v = I18N_EN[s];
  return v !== undefined ? v : s;
}

function browserLang() {
  return (navigator.language || '').toLowerCase().startsWith('zh') ? 'zh' : 'en';
}

function tShowAll(n) {
  return CUR_LANG === 'en' ? 'Show all ' + n : '显示全部 ' + n + ' 条';
}
function tMoreFiles(n) {
  return CUR_LANG === 'en' ? 'Show ' + n + ' more files ⌄' : '再显示 ' + n + ' 个文件 ⌄';
}
function tEditedFiles(n) {
  return CUR_LANG === 'en'
    ? 'Edited ' + n + ' file' + (n > 1 ? 's' : '')
    : '已编辑 ' + n + ' 个文件';
}
function tBound(label) {
  return CUR_LANG === 'en'
    ? 'Bound to ' + label + ' — start a new session to switch'
    : '会话已绑定 ' + label + '，新建会话可切换';
}

function tOpenIn(name) {
  return CUR_LANG === 'en' ? 'Open in ' + name : '在 ' + name + ' 中打开';
}

/** 静态界面文案随语言刷新 */
function applyLang() {
  document.documentElement.setAttribute('lang', CUR_LANG === 'en' ? 'en' : 'zh-CN');
  const S = (sel, txt) => {
    const n = document.querySelector(sel);
    if (n) n.textContent = txt;
  };
  const si = document.querySelector('#search-input');
  if (si) si.placeholder = t('搜索会话');
  S('#btn-new-session', '＋ ' + t('新建会话'));
  S('#btn-new-session-2', t('新建会话'));
  const gp = document.querySelectorAll('#head-projects span');
  if (gp[1]) gp[1].textContent = t('项目');
  const gc = document.querySelectorAll('#head-convs span');
  if (gc[1]) gc[1].textContent = t('对话');
  S('#search-section .group-head span', t('搜索结果'));
  S('#hero-pre', CUR_LANG === 'en' ? 'What can ' : '需要 ');
  S('#hero-post', CUR_LANG === 'en' ? ' do for you?' : ' 帮你做些什么？');
  S('#skill-btn', '／' + t('技能'));
  S('#attach-btn', '📎 ' + t('图片'));
  const loc = document.querySelector('.chip-static');
  if (loc) loc.textContent = '🖥 ' + t('本地');
  S('.hero-tips', t('Enter 发送 · Shift+Enter 换行'));
  const pi = document.querySelector('#prompt-input');
  if (pi) pi.placeholder = state.session ? t('继续这个会话…') : t('输入你的任务…');
  const pb = document.querySelector('#project-btn');
  if (pb && !state.project) pb.textContent = t('选择项目…');
}

function setLang(v) {
  if (v === 'auto') {
    localStorage.removeItem('ah-lang');
    CUR_LANG = browserLang();
  } else {
    localStorage.setItem('ah-lang', v);
    CUR_LANG = v;
  }
  applyLang();
  syncAgentUI();
  renderProjects();
  loadConvs();
  loadStatus();
}

/* ==========================================================================
 * Agent Hub 前端 — 纯原生 JS
 * 结构：工具函数 → agent 配置 → 全局 state → api 封装 → 下拉菜单 →
 *       侧栏渲染 → Hero/输入卡片 → 转录渲染 → NDJSON 流式对话 → 初始化
 * 安全：一切动态文本仅经 textContent / createTextNode 插入，绝不拼接 HTML。
 * ========================================================================== */

/* ---------- 工具函数 ---------- */

const $ = (sel) => document.querySelector(sel);

function el(tag, cls, text) {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text !== undefined && text !== null) n.textContent = text;
  return n;
}

function debounce(fn, ms) {
  let timer = null;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

/** 相对时间：刚刚 / N 分钟前 / N 小时前 / N 天前 / 日期 */
function relTime(iso) {
  if (!iso) return '';
  const t0 = Date.parse(iso);
  if (Number.isNaN(t0)) return '';
  const diff = Date.now() - t0;
  const en = CUR_LANG === 'en';
  if (diff < 60000) return t('刚刚');
  if (diff < 3600000) { const m = Math.floor(diff / 60000); return en ? m + 'm ago' : m + ' 分钟前'; }
  if (diff < 86400000) { const h = Math.floor(diff / 3600000); return en ? h + 'h ago' : h + ' 小时前'; }
  if (diff < 30 * 86400000) { const d = Math.floor(diff / 86400000); return en ? d + 'd ago' : d + ' 天前'; }
  const d = new Date(t0);
  const pad = (x) => String(x).padStart(2, '0');
  return d.getFullYear() + '-' + pad(d.getMonth() + 1) + '-' + pad(d.getDate());
}

/** 单行摘要（折叠卡片标题用） */
function oneLine(t) {
  return (t || '').replace(/\s+/g, ' ').trim().slice(0, 120);
}

/** 截断片段（会话标题用） */
function snippet(t, n) {
  const s = (t || '').replace(/\s+/g, ' ').trim();
  return s.length > n ? s.slice(0, n) + '…' : s;
}

/** 客户端路径粗规范化（仅用于比较，不改动展示值） */
function clientNorm(p) {
  return (p || '').replace(/\//g, '\\').replace(/\\+$/, '').toLowerCase();
}

function projName(path) {
  const parts = (path || '').replace(/[\\/]+$/, '').split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

function folderIcon() {
  const ns = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(ns, 'svg');
  svg.setAttribute('viewBox', '0 0 16 16');
  svg.setAttribute('width', '14');
  svg.setAttribute('height', '14');
  svg.setAttribute('aria-hidden', 'true');
  svg.classList.add('folder-ico');
  const p = document.createElementNS(ns, 'path');
  p.setAttribute(
    'd',
    'M1.75 4.25c0-.83.67-1.5 1.5-1.5h2.69c.4 0 .78.16 1.06.44l1.06 1.06h4.69c.83 0 1.5.67 1.5 1.5v6c0 .83-.67 1.5-1.5 1.5H3.25c-.83 0-1.5-.67-1.5-1.5v-7.5z'
  );
  p.setAttribute('fill', 'currentColor');
  svg.appendChild(p);
  return svg;
}

/** Markdown 渲染：标题/列表/引用/表格/分隔线/代码围栏/行内样式。
 *  全部经 DOM 构建 + textContent 插入，防 XSS。 */
function renderMarkdown(container, raw) {
  container.textContent = '';
  const lines = (raw || '').split('\n');
  let i = 0;
  let paraBuf = [];
  const flushPara = () => {
    if (!paraBuf.length) return;
    const joined = paraBuf.join('\n');
    paraBuf = [];
    if (!joined.trim()) return;
    const p = el('div', 'md-text');
    appendInline(p, joined);
    container.appendChild(p);
  };
  const isTableSep = (s) => /^\s*\|?[\s:\-|]+\|?\s*$/.test(s) && s.includes('-') && s.includes('|');
  const parseRow = (s) =>
    s.replace(/^\s*\|/, '').replace(/\|\s*$/, '').split('|').map((c) => c.trim());

  while (i < lines.length) {
    const line = lines[i];
    // 代码围栏
    if (/^\s*```/.test(line)) {
      flushPara();
      const lang = (line.match(/^\s*```\s*([\w+#.-]+)?/) || [])[1] || '';
      i++;
      const code = [];
      while (i < lines.length && !/^\s*```/.test(lines[i])) {
        code.push(lines[i]);
        i++;
      }
      i++; // 闭栏（流式未闭合时越过末尾，无害）
      container.appendChild(codeBlock(lang, code.join('\n')));
      continue;
    }
    // 标题
    const hm = line.match(/^(#{1,6})\s+(.*)$/);
    if (hm) {
      flushPara();
      const h = el('div', 'md-hd md-h' + Math.min(hm[1].length, 4));
      appendInline(h, hm[2]);
      container.appendChild(h);
      i++;
      continue;
    }
    // 分隔线
    if (/^\s*(?:-{3,}|\*{3,}|_{3,})\s*$/.test(line)) {
      flushPara();
      container.appendChild(el('hr', 'md-hr'));
      i++;
      continue;
    }
    // 引用块
    if (/^\s*>/.test(line)) {
      flushPara();
      const qlines = [];
      while (i < lines.length && /^\s*>/.test(lines[i])) {
        qlines.push(lines[i].replace(/^\s*>\s?/, ''));
        i++;
      }
      const q = el('blockquote', 'md-quote');
      appendInline(q, qlines.join('\n'));
      container.appendChild(q);
      continue;
    }
    // 表格
    if (line.includes('|') && line.trim() && i + 1 < lines.length && isTableSep(lines[i + 1])) {
      flushPara();
      const tbl = el('table', 'md-table');
      const thead = el('thead');
      const trh = el('tr');
      for (const c of parseRow(line)) {
        const th = el('th');
        appendInline(th, c);
        trh.appendChild(th);
      }
      thead.appendChild(trh);
      tbl.appendChild(thead);
      i += 2;
      const tbody = el('tbody');
      while (i < lines.length && lines[i].includes('|') && lines[i].trim()) {
        const tr = el('tr');
        for (const c of parseRow(lines[i])) {
          const td = el('td');
          appendInline(td, c);
          tr.appendChild(td);
        }
        tbody.appendChild(tr);
        i++;
      }
      tbl.appendChild(tbody);
      container.appendChild(tbl);
      continue;
    }
    // 列表（无序/有序，一层嵌套）
    const lm = line.match(/^(\s*)([-*+]|\d+[.)])\s+(.*)$/);
    if (lm) {
      flushPara();
      const ordered = /\d/.test(lm[2]);
      const listEl = el(ordered ? 'ol' : 'ul', 'md-list');
      if (ordered) {
        const start = parseInt(lm[2], 10);
        if (start > 1) listEl.start = start;
      }
      while (i < lines.length) {
        const lm2 = lines[i].match(/^(\s*)([-*+]|\d+[.)])\s+(.*)$/);
        if (!lm2) {
          // 缩进续行并入上一项
          if (/^\s{2,}\S/.test(lines[i]) && listEl.lastElementChild) {
            listEl.lastElementChild.appendChild(document.createTextNode('\n'));
            appendInline(listEl.lastElementChild, lines[i].trim());
            i++;
            continue;
          }
          break;
        }
        const li = el('li');
        appendInline(li, lm2[3]);
        if (lm2[1].length >= 2 && listEl.lastElementChild) {
          let sub = listEl.lastElementChild.querySelector(':scope > ul, :scope > ol');
          if (!sub) {
            sub = el(/\d/.test(lm2[2]) ? 'ol' : 'ul', 'md-list');
            listEl.lastElementChild.appendChild(sub);
          }
          sub.appendChild(li);
        } else {
          listEl.appendChild(li);
        }
        i++;
      }
      container.appendChild(listEl);
      continue;
    }
    if (!line.trim()) {
      flushPara();
      i++;
      continue;
    }
    paraBuf.push(line);
    i++;
  }
  flushPara();
  promoteMarkdownLead(container);
}

/** 把回答开头的首个标题区提升为摘要卡，直到下一标题前为止。
 *  只依赖 Markdown 结构，不绑定“结论 / Summary”等具体文案。
 *  收敛条件（缺一不可，否则整篇正文都会被套进强调卡）：
 *  后面还有同级标题；首段够短；里面没有代码块/表格这类本身带边框的块。 */
const LEAD_MAX_BLOCKS = 3;
const LEAD_BLOCKED = ['codeblock', 'md-table', 'md-pre'];

function promoteMarkdownLead(container) {
  const nodes = [...container.children];
  const first = nodes[0];
  if (!first || !first.classList.contains('md-hd')) return;
  const nextHead = nodes.findIndex((n, i) => i > 0 && n.classList.contains('md-hd'));
  if (nextHead < 2) return; // 首段为空，或整篇只有一个标题区
  const section = nodes.slice(0, nextHead);
  if (section.length > LEAD_MAX_BLOCKS) return;
  if (section.some((n) => LEAD_BLOCKED.some((cls) => n.classList.contains(cls)))) return;

  const lead = el('section', 'md-lead');
  container.insertBefore(lead, first);
  for (const node of section) lead.appendChild(node);
}

/** 代码块卡片：语言标签头部栏 + 复制按钮 */
function codeBlock(lang, codeText) {
  const wrap = el('div', 'codeblock');
  const head = el('div', 'codeblock-head');
  head.appendChild(el('span', 'codeblock-lang', lang || 'text'));
  // HTML 代码块：沙箱 iframe 实时预览
  const isHtml = /^(html?|xml|svg)$/i.test(lang) || /^\s*(<!doctype html|<html)/i.test(codeText);
  if (isHtml) {
    const pv = el('button', 'codeblock-copy', t('▶ 预览'));
    pv.type = 'button';
    let frame = null;
    pv.addEventListener('click', () => {
      if (frame) {
        frame.remove();
        frame = null;
        pv.textContent = t('▶ 预览');
        return;
      }
      frame = el('iframe', 'html-preview');
      frame.setAttribute('sandbox', 'allow-scripts'); // 隔离源：不能访问本应用/本地存储
      frame.srcdoc = codeText;
      wrap.appendChild(frame);
      pv.textContent = t('✕ 收起预览');
    });
    head.appendChild(pv);
  }
  const btn = el('button', 'codeblock-copy', t('⧉ 复制'));
  btn.type = 'button';
  btn.addEventListener('click', () => {
    navigator.clipboard
      .writeText(codeText)
      .then(() => {
        btn.textContent = t('✓ 已复制');
        setTimeout(() => {
          btn.textContent = t('⧉ 复制');
        }, 1500);
      })
      .catch(() => {});
  });
  head.appendChild(btn);
  const pre = el('pre', 'md-pre');
  const c = el('code');
  c.textContent = codeText;
  pre.appendChild(c);
  wrap.appendChild(head);
  wrap.appendChild(pre);
  return wrap;
}

/** 行内样式先识别强调范围，再在范围内解析 `code`。
 *  这样 **模块（`package`）：** 也能完整加粗，不会把星号裸露出来。 */
function appendInline(node, text) {
  styleInline(node, text);
}

/** 行内反引号：`code` → <code>；未闭合的反引号按字面量还原；
 *  普通文本段再做链接化（markdown 链接 / 裸本地路径 / URL）。 */
function appendCodeInline(node, text) {
  const parts = text.split('`');
  for (let k = 0; k < parts.length; k++) {
    if (k % 2 === 1) {
      if (k === parts.length - 1) {
        linkify(node, '`' + parts[k]);
      } else {
        const c = el('code', 'md-code');
        c.textContent = parts[k];
        node.appendChild(c);
      }
    } else if (parts[k]) {
      linkify(node, parts[k]);
    }
  }
}

/** 行内样式：**加粗** / __加粗__ / ~~删除线~~（内部继续解析代码和链接） */
function styleInline(node, text) {
  const re = /(\*\*[^\n]+?\*\*|__[^\n]+?__|~~[^\n]+?~~)/g;
  let last = 0;
  let m;
  while ((m = re.exec(text))) {
    if (m.index > last) appendCodeInline(node, text.slice(last, m.index));
    const tok = m[0];
    if (tok.startsWith('~~')) {
      const s = el('del', 'md-del');
      appendCodeInline(s, tok.slice(2, -2));
      node.appendChild(s);
    } else {
      const b = el('strong', 'md-b');
      appendCodeInline(b, tok.slice(2, -2));
      node.appendChild(b);
    }
    last = m.index + tok.length;
  }
  if (last < text.length) appendCodeInline(node, text.slice(last));
}

/** [文本](目标)、裸 Windows 绝对路径、http(s) URL → 可点击链接 */
const LINK_RE =
  /\[([^\]\n]{1,120})\]\(([^)\n]{1,300})\)|((?:[A-Za-z]:[\\/])[^\s"'`（）()，。;；:：<>|*?\n]{2,260}(?::\d{1,6})?)|(https?:\/\/[^\s"'`（）()<>\n]{4,300})/g;

function linkify(node, text) {
  LINK_RE.lastIndex = 0;
  let last = 0;
  let m;
  while ((m = LINK_RE.exec(text))) {
    if (m.index > last) node.appendChild(document.createTextNode(text.slice(last, m.index)));
    if (m[1] !== undefined) node.appendChild(fileLink(m[1], m[2]));
    else if (m[3] !== undefined) node.appendChild(fileLink(m[3], m[3]));
    else node.appendChild(fileLink(m[4], m[4]));
    last = m.index + m[0].length;
  }
  if (last < text.length) node.appendChild(document.createTextNode(text.slice(last)));
}

function fileLink(label, target) {
  const a = el('a', 'file-link', label);
  if (/^https?:\/\//i.test(target)) {
    a.href = target;
    a.target = '_blank';
    a.rel = 'noopener';
    a.title = target;
    return a;
  }
  a.href = '#';
  a.title = target + '\n点击用系统默认程序打开';
  a.addEventListener('click', (e) => {
    e.preventDefault();
    openLocalPath(target);
  });
  return a;
}

async function openLocalPath(path) {
  try {
    await api.post('/api/open', {
      path,
      project: state.session ? state.session.project : state.project,
    });
  } catch (e) {
    alert(t('打开失败：') + e.message);
  }
}

function renderSkeleton(container, n, wide) {
  container.textContent = '';
  for (let i = 0; i < n; i++) {
    const s = el('div', 'skel' + (wide ? ' skel-wide' : ''));
    s.style.width = 55 + ((i * 17) % 40) + '%';
    container.appendChild(s);
  }
}

function errorRow(msg, retry) {
  const d = el('div', 'empty');
  d.appendChild(document.createTextNode(msg + ' '));
  const a = el('button', 'link-btn', t('重试'));
  a.type = 'button';
  a.addEventListener('click', retry);
  d.appendChild(a);
  return d;
}

/* ---------- 官方品牌图标（Simple Icons 提取的 path） ---------- */

const ICON_PATHS = {
  claude: "m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z",
  codex: "M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z",
};

function agentIcon(agent, size) {
  const ns = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(ns, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('width', size);
  svg.setAttribute('height', size);
  svg.setAttribute('aria-hidden', 'true');
  const p = document.createElementNS(ns, 'path');
  p.setAttribute('d', ICON_PATHS[agent] || ICON_PATHS.codex);
  p.setAttribute('fill', 'currentColor');
  svg.appendChild(p);
  return svg;
}

/* ---------- agent 配置 ---------- */

const AGENTS = {
  claude: {
    label: 'Claude Code',
    glyph: '✳',
    cls: 'ag-claude',
    permissions: [
      { value: 'bypass', label: '绕过权限' },
      { value: 'accept-edits', label: '接受编辑' },
      { value: 'default', label: '默认' },
      { value: 'plan', label: '计划' },
    ],
  },
  codex: {
    label: 'Codex',
    glyph: '●',
    cls: 'ag-codex',
    permissions: [
      { value: 'bypass', label: '绕过权限' },
      { value: 'default', label: '工作区可写' },
      { value: 'read-only', label: '只读' },
    ],
  },
};

/* ---------- 全局状态 ---------- */

const state = {
  agent: 'claude',        // 输入卡片当前目标 agent
  permission: 'bypass',   // ChatReq.permission 英文枚举
  model: null,            // null = 默认模型（不传）
  effort: null,           // null = 默认思考等级（不传）
  fast: false,            // Claude 手动快速模式；Codex 始终强制 Fast
  project: null,          // Hero 选中的项目路径
  projects: [],           // /api/projects 结果
  session: null,          // {agent, id|null, project, title}；null = Hero 新会话态
  streaming: false,
  abort: null,            // AbortController
  activeKey: null,        // 侧栏高亮 "agent:id"
  expanded: (() => {
    // 已展开的项目路径（持久化，重开页面保持）
    try {
      return new Set(JSON.parse(localStorage.getItem('ah-expanded') || '[]'));
    } catch (_) {
      return new Set();
    }
  })(),
  searchSeq: 0,           // 搜索请求竞态序号
  skillsCache: {},        // project 规范化路径 → /api/skills 结果
  modelsPromise: null,    // /api/models 结果缓存
  editorsPromise: null,   // /api/editors 已安装编辑器列表缓存
  attachments: [],        // 输入框图片附件 [{path, name}]（已存服务端临时目录）
  agentFilter: localStorage.getItem('ah-agent-filter') || '', // ''=全部 | claude | codex
  sageOn: localStorage.getItem('ah-sage') === '1',            // SAGE 智能路由开关
  sageFailed: null,       // 失败重路由记忆 {task, agents:[]}（成功后清空）
  sageTaskText: null,     // 当前 SAGE 任务的原始输入，失败记忆据此归属
  pendingSage: null,      // 待持久化的路由决策（init 拿到会话 id 即存）
  memOn: localStorage.getItem('ah-mem') === '1',              // OpenViking 记忆插件开关
  teamOn: localStorage.getItem('ah-team') === '1',            // 团队模式（多子 agent 分工）
  teamSize: Math.min(5, Math.max(2, +localStorage.getItem('ah-team-n') || 3)),
  runIds: new Set(),        // 当前前台编排涉及的全部 run id（SAGE 并行阶段可有多个）
  stopping: false,        // 停止请求进行中，防重复点击
  stopRequested: false,   // 区分“用户停止”与“仅断开查看”
  runsIndex: {},          // session_id → {running, ok, error}（侧栏状态标识）
  modelsInfo: null,       // /api/models 解析结果（默认模型/思考强度展示用）
  instanceId: null,       // 后端进程实例；变化时自动刷新到新静态资源
  reloading: false,
};

// SAGE 已包含 SELF / COLLABORATE / HANDOFF 组队决策，不与手动团队模式叠加。
// 兼容历史双开状态：启动时优先保留 SAGE，关闭团队模式。
if (state.sageOn && state.teamOn) {
  state.teamOn = false;
  localStorage.setItem('ah-team', '0');
}

/* ---------- DOM 引用 ---------- */

const composerEl = $('#composer');
const promptInput = $('#prompt-input');
const chatMsgs = $('#chat-msgs');
const chatScrollEl = $('#chat-scroll');

/* ---------- api 封装 ---------- */

const api = {
  async get(path) {
    const resp = await fetch(path);
    const data = await resp.json().catch(() => null);
    if (!resp.ok) throw new Error((data && data.error) || 'HTTP ' + resp.status);
    return data;
  },
  async post(path, body) {
    const resp = await fetch(path, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    const data = await resp.json().catch(() => null);
    if (!resp.ok) throw new Error((data && data.error) || 'HTTP ' + resp.status);
    return data;
  },
};

/* ---------- 下拉菜单 ---------- */

let menuEl = null;
let menuOwner = null;

function closeMenu() {
  if (menuEl) {
    menuEl.remove();
    menuEl = null;
    menuOwner = null;
  }
}

/** items: [{value, label, hint?, checked?}] */
function showMenu(anchor, items, onPick) {
  if (menuEl && menuOwner === anchor) {
    closeMenu(); // 再点一次同一按钮 = 关闭
    return;
  }
  closeMenu();
  const menu = el('div', 'menu');
  for (const it of items) {
    const btn = el('button', 'menu-item' + (it.checked ? ' checked' : ''));
    btn.type = 'button';
    btn.appendChild(el('span', 'menu-label', it.label));
    if (it.tag) btn.appendChild(el('span', 'menu-tag', it.tag));
    if (it.hint) btn.appendChild(el('span', 'menu-hint', it.hint));
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      closeMenu();
      onPick(it);
    });
    menu.appendChild(btn);
  }
  document.body.appendChild(menu);
  const r = anchor.getBoundingClientRect();
  const mw = menu.offsetWidth;
  const mh = menu.offsetHeight;
  const x = Math.max(8, Math.min(r.left, window.innerWidth - mw - 8));
  let y = r.bottom + 6;
  if (y + mh > window.innerHeight - 8) y = Math.max(8, r.top - mh - 6);
  menu.style.left = x + 'px';
  menu.style.top = y + 'px';
  menuEl = menu;
  menuOwner = anchor;
}

/* ---------- 技能选择器 ---------- */

/** 拉取当前项目 + 当前 agent 的技能列表（缓存 Promise，并发安全） */
function fetchSkills() {
  const proj = state.session ? state.session.project : state.project;
  const key = clientNorm(proj || '');
  if (!state.skillsCache[key]) {
    state.skillsCache[key] = api
      .get('/api/skills?' + new URLSearchParams({ project: proj || '' }))
      .catch((e) => {
        delete state.skillsCache[key];
        throw e;
      });
  }
  return state.skillsCache[key].then((list) => list.filter((s) => s.agent === currentAgent()));
}

/** fromSlash=true 表示由输入框键入 / 触发，选中后替换开头的斜杠 token */
function openSkillPicker(anchor, fromSlash) {
  if (menuEl && menuOwner === anchor) {
    closeMenu();
    return;
  }
  closeMenu();
  const menu = el('div', 'menu skill-menu');
  menu.addEventListener('click', (e) => e.stopPropagation());
  const search = el('input', 'skill-search');
  search.type = 'text';
  search.placeholder = t('筛选技能 / 命令…');
  search.autocomplete = 'off';
  search.spellcheck = false;
  const listBox = el('div', 'skill-list');
  listBox.appendChild(el('div', 'empty', t('加载技能中…')));
  menu.appendChild(search);
  menu.appendChild(listBox);
  document.body.appendChild(menu);
  const r = anchor.getBoundingClientRect();
  const mw = menu.offsetWidth;
  const mh = Math.min(menu.offsetHeight, 400);
  const x = Math.max(8, Math.min(r.left, window.innerWidth - mw - 8));
  let y = r.top - mh - 6; // 输入卡片在下方，优先向上弹
  if (y < 8) y = Math.min(r.bottom + 6, window.innerHeight - mh - 8);
  menu.style.left = x + 'px';
  menu.style.top = y + 'px';
  menuEl = menu;
  menuOwner = anchor;
  search.focus();

  let all = [];
  let rows = [];
  let activeIdx = 0;
  const setActive = (idx) => {
    if (!rows.length) return;
    activeIdx = ((idx % rows.length) + rows.length) % rows.length; // 循环滚动
    rows.forEach((r, i) => r.classList.toggle('active', i === activeIdx));
    rows[activeIdx].scrollIntoView({ block: 'nearest' });
  };
  const renderList = () => {
    const q = search.value.trim().toLowerCase();
    listBox.textContent = '';
    rows = [];
    const filtered = all.filter(
      (s) =>
        !q ||
        s.invoke.toLowerCase().includes(q) ||
        s.name.toLowerCase().includes(q) ||
        (s.description || '').toLowerCase().includes(q)
    );
    if (!filtered.length) {
      const msg = all.length
        ? t('没有匹配的技能')
        : currentAgent() === 'codex'
          ? t('未发现 Codex 自定义 prompt（~/.codex/prompts/*.md）')
          : t('未发现技能');
      listBox.appendChild(el('div', 'empty', msg));
      return;
    }
    for (const s of filtered.slice(0, 120)) {
      const btn = el('button', 'menu-item skill-item');
      btn.type = 'button';
      const line = el('div', 'skill-line');
      const inv = el('span', 'skill-invoke', s.invoke);
      inv.style.color = skillColor(s.invoke)[0]; // 与输入框高亮同色
      line.appendChild(inv);
      line.appendChild(el('span', 'skill-src', s.source));
      btn.appendChild(line);
      if (s.description) btn.appendChild(el('span', 'skill-desc', s.description));
      btn.title = s.description || s.invoke;
      btn.addEventListener('click', () => {
        closeMenu();
        insertSkill(s.invoke, fromSlash);
      });
      btn.addEventListener('mousemove', () => {
        const i = rows.indexOf(btn);
        if (i >= 0 && i !== activeIdx) setActive(i);
      });
      rows.push(btn);
      listBox.appendChild(btn);
    }
    setActive(0);
  };
  search.addEventListener('input', renderList);
  search.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActive(activeIdx + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActive(activeIdx - 1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (rows[activeIdx]) rows[activeIdx].click();
    }
  });
  fetchSkills()
    .then((list) => {
      if (menuEl !== menu) return; // 已关闭
      all = list;
      renderList();
    })
    .catch((e) => {
      if (menuEl !== menu) return;
      listBox.textContent = '';
      listBox.appendChild(el('div', 'empty', t('技能加载失败：') + e.message));
    });
}

function insertSkill(invoke, replaceSlash) {
  let rest = promptInput.value;
  if (replaceSlash) rest = rest.replace(/^\/\S*\s?/, '');
  if (rest.startsWith(invoke + ' ') || rest === invoke) {
    promptInput.focus();
    return;
  }
  promptInput.value = invoke + ' ' + rest.replace(/^\s+/, '');
  autoGrow();
  promptInput.focus();
  promptInput.selectionStart = promptInput.selectionEnd = promptInput.value.length;
}

/* ---------- 图片附件 ---------- */

async function uploadImage(blob, name) {
  const resp = await fetch('/api/upload?' + new URLSearchParams({ name: name || 'paste.png' }), {
    method: 'POST',
    body: blob,
  });
  const data = await resp.json().catch(() => null);
  if (!resp.ok) throw new Error((data && data.error) || 'HTTP ' + resp.status);
  return data.path;
}

async function addAttachment(file) {
  try {
    const path = await uploadImage(file, file.name || 'paste.png');
    state.attachments.push({ path, name: file.name || '粘贴图片' });
    renderAttachBar();
    hideComposerError();
  } catch (e) {
    showComposerError(t('图片上传失败：') + e.message);
  }
}

function renderAttachBar() {
  const bar = $('#attach-bar');
  bar.textContent = '';
  bar.classList.toggle('hidden', !state.attachments.length);
  state.attachments.forEach((a, idx) => {
    // 长文本粘贴附件：图标 + 首行预览 + 展开回输入框
    if (a.kind === 'text') {
      const chip = el('div', 'attach-chip attach-text');
      chip.appendChild(el('span', 'attach-text-ico', '📄'));
      const body = el('div', 'attach-text-body');
      body.appendChild(el('div', 'attach-text-name', a.name));
      const expand = el('button', 'attach-text-expand', t('在文本框中显示') + ' ›');
      expand.type = 'button';
      expand.addEventListener('click', () => {
        promptInput.value = (promptInput.value ? promptInput.value + '\n' : '') + a.text;
        state.attachments.splice(idx, 1);
        renderAttachBar();
        autoGrow();
        promptInput.focus();
      });
      body.appendChild(expand);
      chip.appendChild(body);
      const x = el('button', 'attach-x', '×');
      x.type = 'button';
      x.title = t('移除');
      x.addEventListener('click', () => {
        state.attachments.splice(idx, 1);
        renderAttachBar();
      });
      chip.appendChild(x);
      chip.title =
        (CUR_LANG === 'en' ? 'Pasted text, ' : '粘贴文本，') +
        a.text.length + (CUR_LANG === 'en' ? ' chars' : ' 字');
      bar.appendChild(chip);
      return;
    }
    const chip = el('div', 'attach-chip');
    const img = el('img', 'attach-thumb');
    img.src = '/api/file?path=' + encodeURIComponent(a.path);
    img.alt = a.name;
    img.addEventListener('click', () => openLightbox(img.src));
    chip.appendChild(img);
    const x = el('button', 'attach-x', '×');
    x.type = 'button';
    x.title = t('移除图片');
    x.addEventListener('click', () => {
      state.attachments.splice(idx, 1);
      renderAttachBar();
    });
    chip.appendChild(x);
    chip.title = a.path;
    bar.appendChild(chip);
  });
}

/* ---------- 侧栏：CLI 状态 ---------- */

function cliRow(name, st) {
  const row = el('div', 'cli-row');
  const dot = el('span', 'cli-dot ' + (st.installed ? 'ok' : 'bad'), '●');
  row.appendChild(dot);
  row.appendChild(
    el('span', 'cli-name', name + (st.installed ? (st.version ? ' ' + st.version : ' ' + t('已安装')) : ' ' + t('未安装')))
  );
  row.title = st.path || st.error || '';
  return row;
}

async function loadStatus() {
  const box = $('#cli-status');
  box.textContent = '';
  box.appendChild(el('div', 'cli-row dim', t('检测 CLI 中…')));
  try {
    const st = await api.get('/api/status');
    box.textContent = '';
    box.appendChild(cliRow('Claude Code', st.claude));
    box.appendChild(cliRow('Codex', st.codex));
  } catch (e) {
    box.textContent = '';
    const row = el('div', 'cli-row bad-text', t('CLI 状态获取失败'));
    row.title = e.message;
    box.appendChild(row);
  }
}

async function checkServerInstance() {
  if (state.reloading) return;
  try {
    const info = await api.get('/api/instance');
    const nextId = info && info.instance_id;
    if (SageScheduler.shouldReloadInstance(state.instanceId, nextId)) {
      state.reloading = true;
      if (state.session) {
        sessionStorage.setItem('ah-reload-session', JSON.stringify(state.session));
      }
      if (promptInput.value) sessionStorage.setItem('ah-reload-draft', promptInput.value);
      location.reload();
      return;
    }
    if (nextId) {
      state.instanceId = nextId;
      document.documentElement.dataset.instanceId = nextId;
    }
  } catch (_) {
    /* 服务切换的短暂不可达不刷新，下一轮继续检查 */
  }
}

function restoreReloadState() {
  let saved = null;
  try {
    saved = JSON.parse(sessionStorage.getItem('ah-reload-session') || 'null');
  } catch (_) { /* 忽略 */ }
  sessionStorage.removeItem('ah-reload-session');
  const draft = sessionStorage.getItem('ah-reload-draft');
  sessionStorage.removeItem('ah-reload-draft');
  if (draft) {
    promptInput.value = draft;
    autoGrow();
  }
  if (saved && saved.agent && saved.id && saved.project) {
    openSession(saved);
    return true;
  }
  return false;
}

/* ---------- 侧栏：项目 ---------- */

async function loadProjects() {
  const listEl = $('#project-list');
  if (!state.projects.length) renderSkeleton(listEl, 3);
  try {
    state.projects = await api.get('/api/projects');
  } catch (e) {
    listEl.textContent = '';
    listEl.appendChild(errorRow(t('项目加载失败'), loadProjects));
    return;
  }
  renderProjects();
  if (!state.project && state.projects.length) {
    // 优先恢复上次选择的项目
    const saved = localStorage.getItem('ah-project');
    const hit = saved && state.projects.find((p) => clientNorm(p.path) === clientNorm(saved));
    setProject(hit ? hit.path : state.projects[0].path);
  }
  // 项目列表就绪后，定位当前打开的会话（刷新自动重开的场景）
  expandProjectFor(state.session);
}

function getEditors() {
  if (!state.editorsPromise) {
    state.editorsPromise = api
      .get('/api/editors')
      .then((list) => (Array.isArray(list) ? list : []))
      .catch(() => []);
  }
  return state.editorsPromise;
}

async function showProjectMenu(e, project) {
  closeMenu();
  const menu = el('div', 'menu');
  menu.addEventListener('click', (ev) => ev.stopPropagation());
  const loading = el('button', 'menu-item', CUR_LANG === 'en' ? 'Detecting apps…' : '正在检测应用…');
  loading.type = 'button';
  loading.disabled = true;
  menu.appendChild(loading);
  document.body.appendChild(menu);
  menuEl = menu;
  menuOwner = null;

  const position = () => {
    const mw = menu.offsetWidth;
    const mh = menu.offsetHeight;
    menu.style.left = Math.max(8, Math.min(e.clientX, window.innerWidth - mw - 8)) + 'px';
    menu.style.top = Math.max(8, Math.min(e.clientY, window.innerHeight - mh - 8)) + 'px';
  };
  position();

  const editors = await getEditors();
  if (!menu.isConnected) return;
  menu.textContent = '';

  const openWith = (app) =>
    api
      .post('/api/open', { path: project.path, app })
      .catch((er) => alert(t('打开失败：') + er.message));
  const items = editors.map((editor) => ({
    label: tOpenIn(editor.name),
    title: editor.path,
    action: () => openWith(editor.id),
  }));
  items.push(
    { label: t('在终端中打开'), action: () => openWith('terminal') },
    { label: t('用资源管理器打开'), action: () => openWith('explorer') },
    {
      label: t('复制目录路径'),
      action: () => navigator.clipboard.writeText(project.path).catch(() => {}),
    }
  );

  for (const item of items) {
    const btn = el('button', 'menu-item', item.label);
    btn.type = 'button';
    if (item.title) btn.title = item.title;
    btn.addEventListener('click', () => {
      closeMenu();
      item.action();
    });
    menu.appendChild(btn);
  }
  position();
}

function renderProjects() {
  const listEl = $('#project-list');
  listEl.textContent = '';
  if (!state.projects.length) {
    listEl.appendChild(el('div', 'empty', t('还未导入项目，点「＋」从历史中选择')));
    return;
  }
  // 固定按文件夹名排序，不随活跃度变动
  const list = state.projects
    .slice()
    .sort((a, b) =>
      (a.name || a.path).localeCompare(b.name || b.path, 'zh-Hans-CN', {
        numeric: true,
        sensitivity: 'base',
      })
    );
  for (const p of list) {
    const grp = el('div', 'pgroup');
    grp.dataset.path = p.path;
    const row = el('div', 'prow');
    row.appendChild(folderIcon());
    const name = el('span', 'prow-name', p.name || p.path);
    if (!p.exists) name.classList.add('missing');
    row.title = p.path + (p.exists ? '' : '（目录不存在）');
    row.appendChild(name);
    const cnt =
      state.agentFilter === 'claude'
        ? p.claude_sessions
        : state.agentFilter === 'codex'
          ? p.codex_sessions
          : p.claude_sessions + p.codex_sessions;
    row.appendChild(el('span', 'prow-count', String(cnt)));
    const caret = el('span', 'prow-caret', '▸');
    if (state.expanded.has(p.path)) caret.classList.add('open');
    row.appendChild(caret);
    row.addEventListener('click', () => toggleProject(p.path));
    row.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      e.stopPropagation();
      showProjectMenu(e, p);
    });
    grp.appendChild(row);
    if (state.expanded.has(p.path)) {
      const box = el('div', 'proj-sessions');
      grp.appendChild(box);
      fillProjectSessions(box, p.path);
    }
    listEl.appendChild(grp);
  }
}

function saveExpanded() {
  localStorage.setItem('ah-expanded', JSON.stringify([...state.expanded]));
}

function toggleProject(path) {
  if (state.expanded.has(path)) state.expanded.delete(path);
  else state.expanded.add(path);
  saveExpanded();
  renderProjects();
}

async function fillProjectSessions(box, path) {
  renderSkeleton(box, 2);
  try {
    await ensureRunsIndex();
    const sessions = await api.get(
      '/api/sessions?' + new URLSearchParams({ project: path, limit: '500' })
    );
    renderSessionList(box, mergeRunningSessions(sessions, path), path);
    const act = box.querySelector('.srow.active');
    if (act) act.scrollIntoView({ block: 'nearest' });
  } catch (e) {
    box.textContent = '';
    box.appendChild(el('div', 'empty', '加载失败：' + e.message));
  }
}

async function addProject(setCurrent) {
  let path = null;
  try {
    const picked = await api.post('/api/pick-folder', {});
    if (!picked || !picked.path) return; // 用户在系统对话框中取消
    path = picked.path;
  } catch (e) {
    if (String(e.message).includes('已有选择窗口')) {
      alert(e.message);
      return;
    }
    // 系统对话框不可用（非 Windows 等）时回退手动输入
    path = prompt(t('输入项目目录的完整路径'));
  }
  if (!path || !path.trim()) return;
  const trimmed = path.trim();
  try {
    const list = await api.post('/api/projects', { path: trimmed });
    if (Array.isArray(list)) state.projects = list;
    renderProjects();
    if (setCurrent) setProject(trimmed);
  } catch (e) {
    alert(t('添加项目失败：') + e.message);
  }
}

function setProject(path) {
  const found = state.projects.find((p) => clientNorm(p.path) === clientNorm(path));
  state.project = found ? found.path : path;
  localStorage.setItem('ah-project', state.project);
  const btn = $('#project-btn');
  btn.textContent = projName(state.project) + ' ⌄';
  btn.title = state.project;
}

/* ---------- 项目导入面板（历史发现的项目，自选导入/移除） ---------- */

async function openProjectPicker(anchor) {
  if (menuEl && menuOwner === anchor) {
    closeMenu();
    return;
  }
  closeMenu();
  const menu = el('div', 'menu skill-menu');
  menu.addEventListener('click', (e) => e.stopPropagation());
  const search = el('input', 'skill-search');
  search.type = 'text';
  search.placeholder = t('搜索历史项目（名称 / 路径）…');
  search.autocomplete = 'off';
  const listBox = el('div', 'skill-list');
  listBox.appendChild(el('div', 'empty', t('扫描历史项目中…')));
  const manual = el('button', 'menu-item', '📁 浏览文件夹导入…');
  manual.type = 'button';
  manual.addEventListener('click', () => {
    closeMenu();
    addProject(!state.project);
  });
  menu.appendChild(search);
  menu.appendChild(listBox);
  menu.appendChild(manual);
  document.body.appendChild(menu);
  const r = anchor.getBoundingClientRect();
  const mw = menu.offsetWidth;
  const mh = Math.min(menu.offsetHeight, 420);
  const x = Math.max(8, Math.min(r.left, window.innerWidth - mw - 8));
  let y = r.bottom + 6;
  if (y + mh > window.innerHeight - 8) y = Math.max(8, r.top - mh - 6);
  menu.style.left = x + 'px';
  menu.style.top = y + 'px';
  menuEl = menu;
  menuOwner = anchor;
  search.focus();

  let all = [];
  const render = () => {
    const q = search.value.trim().toLowerCase();
    listBox.textContent = '';
    const filtered = all.filter(
      (p) => !q || (p.name || '').toLowerCase().includes(q) || (p.path || '').toLowerCase().includes(q)
    );
    if (!filtered.length) {
      listBox.appendChild(el('div', 'empty', all.length ? t('没有匹配的项目') : t('历史中没有发现项目')));
      return;
    }
    for (const p of filtered.slice(0, 200)) {
      const row = el('div', 'pick-row');
      const info = el('div', 'pick-info');
      const line = el('div', 'skill-line');
      line.appendChild(el('span', 'skill-name', p.name || p.path));
      line.appendChild(
        el('span', 'skill-src', p.claude_sessions + p.codex_sessions + ' ' + t('会话') + ' · ' + relTime(p.last_active))
      );
      info.appendChild(line);
      info.appendChild(el('span', 'skill-desc', p.path));
      row.appendChild(info);
      const btn = el('button', 'pick-btn' + (p.pinned ? ' on' : ''), p.pinned ? t('移除') : t('导入'));
      btn.type = 'button';
      btn.addEventListener('click', async () => {
        btn.disabled = true;
        try {
          const list = await api.post(p.pinned ? '/api/projects/remove' : '/api/projects', {
            path: p.path,
          });
          if (Array.isArray(list)) state.projects = list;
          p.pinned = !p.pinned;
          renderProjects();
          loadConvs();
          if (p.pinned && !state.project) setProject(p.path);
          render();
        } catch (e) {
          alert(t('操作失败：') + e.message);
          btn.disabled = false;
        }
      });
      row.appendChild(btn);
      listBox.appendChild(row);
    }
  };
  search.addEventListener('input', render);
  try {
    all = await api.get('/api/projects/discover');
    if (menuEl !== menu) return;
    render();
  } catch (e) {
    if (menuEl !== menu) return;
    listBox.textContent = '';
    listBox.appendChild(el('div', 'empty', t('扫描失败：') + e.message));
  }
}

/* ---------- 侧栏：会话列表与搜索 ---------- */

/** 分支头部：收起/展开箭头 + 收起时的子会话计数。
 *  箭头吃掉点击，否则点开合会顺手打开这条会话。 */
function addBranchToggle(row, branchKey, count, collapsed) {
  row.dataset.branchHead = branchKey;
  const badge = el('span', 'srow-subcount' + (collapsed ? '' : ' hidden'), String(count));
  row.appendChild(badge);
  const caret = el('span', 'srow-caret' + (collapsed ? '' : ' open'), '▸');
  caret.title = tBranch(count);
  caret.addEventListener('click', (e) => {
    e.stopPropagation();
    toggleBranch(branchKey);
  });
  row.appendChild(caret);
}

function tBranch(count) {
  return CUR_LANG === 'en'
    ? `${count} handoff sub-session${count > 1 ? 's' : ''} — click to collapse/expand`
    : `${count} 个移交子会话 · 点击收起/展开`;
}

/** nested=true：SAGE 移交子会话，缩进挂在来源会话下 */
function sessionRow(s, nested) {
  const cfg = AGENTS[s.agent];
  const row = el('div', 'srow' + (nested ? ' srow-sub' : '') + (s.archived ? ' archived' : ''));
  row.dataset.key = s.agent + ':' + s.id;
  if (state.activeKey === row.dataset.key) row.classList.add('active');
  const dot = el('span', 'agent-dot ' + (cfg ? cfg.cls : 'ag-codex'));
  dot.appendChild(agentIcon(s.agent, 12));
  row.appendChild(dot);
  row.appendChild(el('span', 'srow-title', s.title || t('(无标题)')));
  applyRunBadge(row, state.runsIndex[s.id]);
  const timeEl = el('span', 'srow-time', relTime(s.updated || s.created));
  timeEl.dataset.ts = s.updated || s.created || ''; // 供定时器原地刷新相对时间
  row.appendChild(timeEl);
  row.title = (s.title || t('(无标题)')) + '\n' + (s.project || '') +
    (nested ? '\n' + t('（SAGE 移交子会话）') : '') + (s.archived ? '\n（已归档）' : '');
  row.addEventListener('click', () => openSession(s));
  // 右键：删除会话（移入 ~/.agenthub/trash 回收目录，可恢复）
  row.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    e.stopPropagation();
    showMenu(row, [{ value: 'del', label: t('🗑 删除会话') }], async (it) => {
      if (it.value !== 'del') return;
      const r = state.runsIndex[s.id];
      if (r && r.running) {
        showToast(t('会话有任务在运行，先停止再删除'));
        return;
      }
      if (!confirm(t('确定删除该会话？文件将移入回收目录（可恢复）'))) return;
      try {
        await api.post('/api/session/delete', { agent: s.agent, id: s.id });
        if (state.session && state.session.id === s.id) onNewSession();
        loadConvs();
        loadProjects();
        renderProjects();
        showToast(t('已删除'));
      } catch (e2) {
        alert(t('删除失败：') + e2.message);
      }
    });
  });
  return row;
}

/** runs → session_id 索引。同一会话可能有多次运行（旧轮已完成 + 新轮运行中，
 *  注册表顺序随机）：运行中优先，其次取较新的（run_id 含毫秒时间戳）。 */
function buildRunsIndex(runs) {
  const idx = SageScheduler.buildRunsIndex(runs);
  for (const r of runs || []) {
    const link = SageScheduler.collabLinkFromRun(r);
    if (link) collabLinkSave(link.primaryKey, link.entry, link.partnerKey);
    if (r.running) seenRunningPartners.add(r.session_id); // 供自动回注判定
  }
  return idx;
}

/** 运行注册表兜底：刷新 runsIndex（列表合并运行中任务用，3 秒内不重复拉） */
let runsIndexAt = 0;
async function ensureRunsIndex() {
  if (Date.now() - runsIndexAt < 3000) return;
  try {
    const runs = await api.get('/api/runs');
    state.runsIndex = buildRunsIndex(runs);
    runsIndexAt = Date.now();
  } catch (_) {
    /* 忽略 */
  }
}

/** 会话列表合并运行中任务：新会话文件未落盘/未被索引时（claude 首轮长任务
 *  尤其明显），列表会缺席几分钟——用注册表数据合成占位行，避免误以为没发出去 */
function mergeRunningSessions(sessions, project) {
  const enriched = sessions.map((session) => {
    const run = state.runsIndex[session.id];
    if (
      run && run.display_prompt &&
      (!session.title || session.title === '(无标题)' || session.title === t('(无标题)'))
    ) {
      return { ...session, title: snippet(run.display_prompt, 40) };
    }
    return session;
  });
  const have = new Set(enriched.map((s) => s.agent + ':' + s.id));
  const extra = [];
  for (const [sid, r] of Object.entries(state.runsIndex)) {
    if (!r.running || !sid || !r.agent) continue;
    if (project && clientNorm(r.project || '') !== clientNorm(project)) continue;
    if (have.has(r.agent + ':' + sid)) continue;
    extra.push({
      agent: r.agent,
      id: sid,
      title: snippet(r.display_prompt || r.prompt || '(运行中)', 40),
      project: r.project || '',
      created: null,
      updated: new Date().toISOString(),
      archived: false,
    });
  }
  return extra.concat(enriched);
}

async function loadConvs() {
  const listEl = $('#conv-list');
  if (!listEl.childElementCount) renderSkeleton(listEl, 4);
  try {
    await ensureRunsIndex();
    const sessions = await api.get('/api/sessions?limit=30');
    renderSessionList(listEl, mergeRunningSessions(sessions, null), 'convs');
  } catch (e) {
    listEl.textContent = '';
    listEl.appendChild(errorRow(t('会话加载失败'), loadConvs));
  }
}

async function onSearch() {
  const q = $('#search-input').value.trim();
  const seq = ++state.searchSeq;
  if (!q) {
    $('#search-section').classList.add('hidden');
    $('#browse-section').classList.remove('hidden');
    return;
  }
  $('#search-section').classList.remove('hidden');
  $('#browse-section').classList.add('hidden');
  const listEl = $('#search-list');
  renderSkeleton(listEl, 3);
  try {
    const raw = await api.get('/api/sessions?' + new URLSearchParams({ q, limit: '50' }));
    if (seq !== state.searchSeq) return; // 已有更新的搜索
    const sessions = filterSessions(raw);
    listEl.textContent = '';
    if (!sessions.length) {
      listEl.appendChild(el('div', 'empty', t('没有匹配的会话')));
      return;
    }
    for (const s of sessions) listEl.appendChild(sessionRow(s));
  } catch (e) {
    if (seq !== state.searchSeq) return;
    listEl.textContent = '';
    listEl.appendChild(el('div', 'empty', t('搜索失败：') + e.message));
  }
}

/* ---------- 分组收缩与「显示全部」 ---------- */

/** 各列表当前显示条数（key：'convs' 或项目路径），持久化重开保持 */
const listShown = (() => {
  try {
    return new Map(Object.entries(JSON.parse(localStorage.getItem('ah-list-shown') || '{}')));
  } catch (_) {
    return new Map();
  }
})();

function saveListShown() {
  localStorage.setItem('ah-list-shown', JSON.stringify(Object.fromEntries(listShown)));
}
const LIST_BASE = 6;
const LIST_STEP = 20;

function tShowMore(step, remaining) {
  return CUR_LANG === 'en'
    ? 'Show ' + step + ' more (' + remaining + ' left)'
    : '再显示 ' + step + ' 条（剩 ' + remaining + '）';
}

/** 收起的移交分支（存父会话 key）。默认展开，只记用户手动收起过的。 */
function collapsedBranches() {
  try {
    return new Set(JSON.parse(localStorage.getItem('ah-branch-collapsed') || '[]'));
  } catch (_) {
    return new Set();
  }
}

function toggleBranch(branchKey) {
  const set = collapsedBranches();
  if (set.has(branchKey)) set.delete(branchKey);
  else set.add(branchKey);
  localStorage.setItem('ah-branch-collapsed', JSON.stringify([...set]));
  applyBranchCollapse();
}

/** 同一分支可能同时出现在「对话」和项目列表里，按 key 一起同步，不重新拉数据 */
function applyBranchCollapse() {
  const collapsed = collapsedBranches();
  document.querySelectorAll('.srow[data-branch]').forEach((row) => {
    // 正在查看的子会话不跟着藏，否则当前会话在侧栏没了着落
    const active = state.activeKey === row.dataset.key;
    row.classList.toggle('hidden', collapsed.has(row.dataset.branch) && !active);
  });
  document.querySelectorAll('.srow[data-branch-head]').forEach((row) => {
    const off = collapsed.has(row.dataset.branchHead);
    const caret = row.querySelector('.srow-caret');
    if (caret) caret.classList.toggle('open', !off);
    const count = row.querySelector('.srow-subcount');
    if (count) count.classList.toggle('hidden', !off);
  });
}

function collapsedGroups() {
  try {
    return new Set(JSON.parse(localStorage.getItem('ah-collapsed') || '[]'));
  } catch (_) {
    return new Set();
  }
}

function toggleGroup(key) {
  const set = collapsedGroups();
  if (set.has(key)) set.delete(key);
  else set.add(key);
  localStorage.setItem('ah-collapsed', JSON.stringify([...set]));
  applyGroupCollapse();
}

function applyGroupCollapse() {
  const set = collapsedGroups();
  const groups = [
    ['projects', '#head-projects', '#project-list'],
    ['convs', '#head-convs', '#conv-list'],
  ];
  for (const [key, headSel, listSel] of groups) {
    const off = set.has(key);
    $(listSel).classList.toggle('hidden', off);
    const caret = $(headSel).querySelector('.group-caret');
    if (caret) caret.textContent = off ? '▸' : '▾';
  }
}

/** 侧栏 agent 过滤（左侧图标栏） */
function filterSessions(sessions) {
  // 协作子会话不进侧栏（在主会话右侧面板展示状态与入口）
  for (const link of SageScheduler.collabLinksFromSessions(sessions)) {
    collabLinkSave(link.primaryKey, link.entry, link.partnerKey);
  }
  const store = collabStoreLoad();
  sessions = sessions.filter(
    (s) =>
      !SageScheduler.isNestedSageSession(s, state.runsIndex, store) &&
      !/^【协作(分工|复查|追问)】/.test(s.title || '')
  );
  if (!state.agentFilter) return sessions;
  return sessions.filter((s) => s.agent === state.agentFilter);
}

function setAgentFilter(f) {
  state.agentFilter = f;
  localStorage.setItem('ah-agent-filter', f);
  document.querySelectorAll('.rail-btn[data-filter]').forEach((b) => {
    b.classList.toggle('active', (b.dataset.filter || '') === f);
  });
  // 新会话的默认 agent 跟随过滤选择
  if (f && canSwitchAgent() && state.agent !== f) setAgent(f);
  renderProjects();
  loadConvs();
  if ($('#search-input').value.trim()) onSearch();
}

/** 会话列表：默认 6 条，点击阶梯式追加 20 条（新增行瀑布渐入），展开后可收起。
 *  条数按顶层分支计，移交子会话跟随来源会话一起显示，不单独占配额。 */
function renderSessionList(listEl, sessions, key, animateFrom) {
  sessions = filterSessions(sessions);
  listEl.textContent = '';
  if (!sessions.length) {
    listEl.appendChild(el('div', 'empty', t('暂无对话')));
    return;
  }
  // filterSessions 会先把回溯配对的关联写回本地表，这里再读才拿得到最新父子关系
  const tree = SageScheduler.buildSessionTree(sessions, state.runsIndex, collabStoreLoad());
  const collapsed = collapsedBranches();
  const shown = Math.min(listShown.get(key) || LIST_BASE, tree.length);
  for (let i = 0; i < shown; i++) {
    const node = tree[i];
    const head = sessionRow(node.session);
    const rows = [head];
    if (node.children.length) {
      const branchKey = node.session.agent + ':' + node.session.id;
      // 正在查看的子会话所在分支强制展开，否则当前会话在侧栏找不到
      const off =
        collapsed.has(branchKey) &&
        !node.children.some((c) => state.activeKey === c.agent + ':' + c.id);
      addBranchToggle(head, branchKey, node.children.length, off);
      for (const child of node.children) {
        const row = sessionRow(child, true);
        row.dataset.branch = branchKey;
        if (off) row.classList.add('hidden');
        rows.push(row);
      }
    }
    for (const row of rows) {
      if (animateFrom !== undefined && i >= animateFrom) {
        row.classList.add('row-in');
        row.style.animationDelay = Math.min((i - animateFrom) * 25, 500) + 'ms';
      }
      listEl.appendChild(row);
    }
  }
  const foot = el('div', 'list-foot');
  if (shown < tree.length) {
    const remaining = tree.length - shown;
    const more = el('button', 'link-btn more-btn', tShowMore(Math.min(LIST_STEP, remaining), remaining));
    more.type = 'button';
    more.addEventListener('click', () => {
      listShown.set(key, shown + LIST_STEP);
      saveListShown();
      renderSessionList(listEl, sessions, key, shown);
    });
    foot.appendChild(more);
  }
  if (shown > LIST_BASE) {
    const less = el('button', 'link-btn more-btn', t('收起'));
    less.type = 'button';
    less.addEventListener('click', () => {
      listShown.set(key, LIST_BASE);
      saveListShown();
      renderSessionList(listEl, sessions, key);
    });
    foot.appendChild(less);
  }
  if (foot.childElementCount) listEl.appendChild(foot);
}

/* ---------- 会话运行状态标识（运行中 / 已完成 / 报错） ---------- */

function applyRunBadge(row, st) {
  let b = row.querySelector('.run-badge');
  if (!st) {
    if (b) b.remove();
    return;
  }
  if (!b) {
    b = el('span', 'run-badge');
    const time = row.querySelector('.srow-time');
    row.insertBefore(b, time || null);
  }
  const stopped = !st.running && st.error === '已停止';
  const cls = st.running ? 'running' : stopped ? 'stopped' : st.ok ? 'ok' : 'err';
  b.className = 'run-badge ' + cls;
  b.textContent = st.running ? '●' : stopped ? '■' : st.ok ? '✓' : '✕';
  b.title = st.running
    ? t('运行中')
    : stopped ? t('■ 已停止') : st.ok ? t('已完成') : t('运行出错：') + (st.error || t('未知错误'));
}

function refreshRunBadges() {
  const links = collabStoreLoad().links;
  document.querySelectorAll('.srow').forEach((row) => {
    const key = row.dataset.key || '';
    const id = key.split(':').slice(1).join(':');
    let r = state.runsIndex[id];
    // 聚合协作子会话状态：主会话自身已结束但分工子会话还在跑 → 整体仍显示运行中
    if (!r || !r.running) {
      for (const ln of links[key] || []) {
        const pr = state.runsIndex[ln.partner.slice(ln.partner.indexOf(':') + 1)];
        if (pr && pr.running) {
          r = { running: true };
          break;
        }
      }
    }
    applyRunBadge(row, r);
  });
}

let lastRunningSig = '';
async function pollRuns() {
  try {
    const runs = await api.get('/api/runs');
    state.runsIndex = buildRunsIndex(runs);
    runsIndexAt = Date.now();
    refreshRunBadges();
    renderCollabPanel(); // 子会话运行状态实时刷新
    // 运行集合变化（新任务开始/结束）→ 重载侧栏列表，运行中会话实时出现
    const sig = runs
      .filter((r) => r.running)
      .map((r) => r.session_id || r.run_id)
      .sort()
      .join(',');
    if (sig !== lastRunningSig) {
      lastRunningSig = sig;
      loadConvs();
      loadProjects(); // 会话计数也跟着变
      renderProjects();
      // 分工子会话状态变化（如跑完）→ 重缝合当前会话（补回注按钮及时浮现）
      if (state.session && state.session.id && !state.streaming) {
        stitchCollab(state.session, null);
      }
    }
  } catch (_) {
    /* 忽略 */
  }
}

function setActiveRow(key) {
  state.activeKey = key;
  document.querySelectorAll('.srow').forEach((r) => {
    r.classList.toggle('active', !!key && r.dataset.key === key);
  });
  applyBranchCollapse(); // 从正文分隔条跳进子会话时，把收起的那条分支放出来
  const act = document.querySelector('.srow.active');
  if (act) act.scrollIntoView({ block: 'nearest' });
}

/** 侧栏自动定位：展开当前会话所属的项目分组 */
function expandProjectFor(sess) {
  if (!sess || !sess.project || !state.projects.length) return;
  const hit = state.projects.find((p) => clientNorm(p.path) === clientNorm(sess.project));
  if (hit && !state.expanded.has(hit.path)) {
    state.expanded.add(hit.path);
    saveExpanded();
    renderProjects();
  }
}

/** 新会话拿到 session_id 后，把会话临时加入「对话」分组（done 后会整体刷新） */
function prependConvRow() {
  const s = state.session;
  if (!s || !s.id) return;
  const listEl = $('#conv-list');
  const empty = listEl.querySelector('.empty');
  if (empty) empty.remove();
  listEl.prepend(
    sessionRow({
      agent: s.agent,
      id: s.id,
      title: s.title,
      project: s.project,
      created: null,
      updated: new Date().toISOString(),
      archived: false,
    })
  );
  setActiveRow(s.agent + ':' + s.id);
  syncAgentUI(); // 会话已落盘，锁定 agent 切换
}

/* ---------- agent / 权限 / 模型联动 ---------- */

/* 每个 agent 的权限/模型/思考/快速选择持久化（刷新后恢复） */

function savePrefs() {
  let all = {};
  try {
    all = JSON.parse(localStorage.getItem('ah-prefs') || '{}');
  } catch (_) { /* 忽略坏数据 */ }
  all[state.agent] = {
    permission: state.permission,
    model: state.model,
    effort: state.effort,
    fast: state.fast,
  };
  localStorage.setItem('ah-prefs', JSON.stringify(all));
}

function loadAgentPrefs(agent) {
  let all = {};
  try {
    all = JSON.parse(localStorage.getItem('ah-prefs') || '{}');
  } catch (_) { /* 忽略坏数据 */ }
  const p = all[agent] || {};
  state.permission = p.permission || 'bypass';
  state.model = p.model !== undefined ? p.model : null;
  state.effort = p.effort !== undefined ? p.effort : null;
  state.fast = agent === 'codex' ? true : !!p.fast;
}

function setAgent(agent) {
  state.agent = agent;
  loadAgentPrefs(agent); // 恢复该 agent 记住的选择，而不是重置默认
  syncAgentUI();
}

/** 思考等级中文名 */
const EFFORT_LABELS = {
  minimal: '最小',
  low: '低',
  medium: '中',
  high: '高',
  xhigh: '超高',
  max: '最大',
  ultra: 'Ultra',
};

function effortLabel() {
  const pre = CUR_LANG === 'en' ? 'Effort·' : '思考·';
  if (state.sageOn) {
    const active = state.session && state.session.effort;
    return pre + (active
      ? (CUR_LANG === 'en' ? 'Auto·' : '自动·') + t(EFFORT_LABELS[active] || active)
      : CUR_LANG === 'en' ? 'Auto' : '自动');
  }
  if (state.effort !== null) return pre + t(EFFORT_LABELS[state.effort] || state.effort);
  const info = state.modelsInfo && state.modelsInfo[currentAgent()];
  const de = info && info.default_effort;
  return pre + (de ? t(EFFORT_LABELS[de] || de) : t('默认'));
}

/** 会话一旦有 id（历史打开或新会话已落盘），agent 即锁定不可切换 */
function canSwitchAgent() {
  return !state.session || !state.session.id;
}

function currentAgent() {
  return state.session ? state.session.agent : state.agent;
}

/** Codex 的所有模型固定走 Fast；Claude 继续使用用户开关。 */
function fastForAgent(agent) {
  return agent === 'codex' ? true : state.fast;
}

function syncAgentUI() {
  const cfg = AGENTS[state.agent];
  const em = $('#agent-switch');
  em.textContent = cfg.label;
  em.appendChild(el('span', 'caret', ' ⌄'));
  setBadge($('#composer-agent'), currentAgent());
  const badge = $('#composer-agent');
  if (canSwitchAgent()) {
    badge.classList.remove('locked');
    badge.appendChild(el('span', 'caret', ' ⌄'));
    badge.title = t('切换 Agent');
  } else {
    badge.classList.add('locked');
    badge.title = tBound(AGENTS[currentAgent()].label);
  }
  $('#perm-btn').textContent = permLabel();
  const effortBtn = $('#effort-btn');
  effortBtn.textContent = effortLabel();
  effortBtn.title = state.sageOn
    ? '思考强度由 SAGE 按任务复杂度、需求节点和模型支持范围自动选择'
    : '思考强度';
  const mb = $('#model-btn');
  mb.textContent = modelLabel();
  mb.title = '模型：' + modelFull();
  // SAGE 智能路由：新会话 = 选执行者；绑定的协作会话 = 追问分诊（开关同一个）
  const sageBtn = $('#sage-btn');
  const hasCollab = !!(
    state.session &&
    state.session.id &&
    (collabStoreLoad().links[state.session.agent + ':' + state.session.id] || []).length
  );
  sageBtn.classList.toggle('hidden', !canSwitchAgent() && !hasCollab);
  // 已经路由过的会话不再重新决策，按钮上标出来，免得以为智能路由失灵了
  const sageLocked = !!(
    state.sageOn && state.session && state.session.id &&
    sageStoreLoad()[state.session.agent + ':' + state.session.id]
  );
  sageBtn.title = sageLocked
    ? '本会话的执行者已在首轮定下（' + (modelLabel() || currentAgent()) + '），追问一律沿用。\n' +
      '每轮重新路由会被少量学习样本带偏，导致频繁换模型、上下文反复重建。\n' +
      '需要换执行者：手动切换，或新建会话重新路由；执行失败时会自动重路由。'
    : canSwitchAgent()
      ? 'SAGE 智能路由：按任务复杂度自动选择团队人数、CLI、模型、思考强度与需求分工；与团队模式互斥'
      : 'SAGE 追问分诊：属搭档擅长域的追问自动转子会话执行并回注';
  sageBtn.classList.toggle('on', state.sageOn);
  sageBtn.classList.toggle('locked-decision', sageLocked);
  setToggleChip(sageBtn, t(sageLocked ? '🧭 已定执行者' : '🧭 智能路由'), state.sageOn);
  // 快速开关（两家都是真实机制，与思考等级相互独立）：
  // claude = --settings fastMode；codex = -c service_tier（TUI /fast 同款配置键）
  // OpenViking 记忆：claude / codex 均支持（插件 hooks 按次经环境变量开关）
  const memBtn = $('#mem-btn');
  if (memBtn) {
    memBtn.classList.remove('hidden');
    memBtn.classList.toggle('on', state.memOn);
    setToggleChip(memBtn, t('🧠 记忆'), state.memOn);
  }
  // 团队模式：两家都支持（claude=Agent+SendMessage，codex=spawn_agent）
  const teamBtn = $('#team-btn');
  if (teamBtn) {
    teamBtn.title = '团队模式：让 agent 派生多个子 agent 分工协作；与智能路由互斥';
    teamBtn.classList.toggle('on', state.teamOn);
    setToggleChip(teamBtn, t('👥 团队') + (state.teamOn ? ' ' + state.teamSize : ''), state.teamOn);
  }
  const fastBtn = $('#fast-btn');
  const fastForced = currentAgent() === 'codex';
  const fastOn = fastForAgent(currentAgent());
  fastBtn.classList.remove('hidden');
  fastBtn.classList.toggle('on', fastOn);
  setToggleChip(fastBtn, t('⚡ 快速'), fastOn);
  fastBtn.title =
    fastForced
      ? 'Codex 所有模型固定使用 Fast 模式（service_tier=fast），不可关闭'
      : currentAgent() === 'claude'
      ? '快速模式：以 fastMode 设置运行（需模型支持）'
      : '快速模式';
}

/** 滑动开关样式的 chip：左侧文案 + 右侧滑轨圆钮 */
function setToggleChip(btn, label, on) {
  btn.textContent = '';
  btn.appendChild(el('span', 'tg-label', label));
  const track = el('span', 'tg-track' + (on ? ' on' : ''));
  track.appendChild(el('span', 'tg-knob'));
  btn.appendChild(track);
}

function setBadge(target, agent) {
  const cfg = AGENTS[agent];
  target.textContent = '';
  const bdot = el('span', 'agent-dot ' + cfg.cls);
  bdot.appendChild(agentIcon(agent, 14));
  target.appendChild(bdot);
  target.appendChild(document.createTextNode(cfg.label));
}

function permLabel() {
  const f = AGENTS[state.agent].permissions.find((p) => p.value === state.permission);
  return t(f ? f.label : state.permission);
}

function modelFull() {
  if (state.model !== null) return state.model;
  const info = state.modelsInfo && state.modelsInfo[currentAgent()];
  return info && info.default ? info.default : t('默认模型');
}

function modelLabel() {
  // 长模型名截断展示，完整名放悬停提示（syncAgentUI 里设置 title）
  const full = modelFull();
  const short = full.length > 14 ? full.slice(0, 12) + '…' : full;
  return short; // 下拉箭头由 CSS ::after 统一渲染
}

/** /api/models 结果缓存（Promise，失败重试） */
function getModels() {
  if (!state.modelsPromise) {
    state.modelsPromise = api.get('/api/models').catch((e) => {
      state.modelsPromise = null;
      throw e;
    });
  }
  return state.modelsPromise;
}

/* ---------- 视图切换 ---------- */

function showHero() {
  state.session = null;
  state.histUsage = null;
  syncBackPrimaryUI(null); // 从子会话新建时清掉来源关系，否则新会话会被当成子会话禁止移交
  $('#view-chat').classList.add('hidden');
  $('#view-hero').classList.remove('hidden');
  $('#hero-slot').appendChild(composerEl);
  promptInput.placeholder = t('输入你的任务…');
  hideComposerError();
  setActiveRow(null);
  syncAgentUI(); // 解锁 agent 切换
  promptInput.focus();
}

function showChat() {
  $('#view-hero').classList.add('hidden');
  $('#view-chat').classList.remove('hidden');
  $('#chat-slot').appendChild(composerEl);
}

/** 停止后台运行（只有这里会杀 CLI 进程） */
async function stopRun() {
  if (state.stopping) return;
  state.stopping = true;
  setSendButton(true);
  showToast(CUR_LANG === 'en' ? 'Stopping task…' : '正在停止任务…');
  const localIds = [...state.runIds];
  if (state.runId) localIds.push(state.runId);
  const key = state.session && state.session.id
    ? state.session.agent + ':' + state.session.id
    : '';
  const storedDecision = key ? sageStoreLoad()[key] || null : null;
  const links = key ? collabStoreLoad().links[key] || [] : [];
  const partnerSessionIds = links.map((link) =>
    link.partner.slice(link.partner.indexOf(':') + 1)
  );
  const workflowIds = [
    state.pendingSage && state.pendingSage.workflow_id,
    storedDecision && storedDecision.workflow_id,
  ].filter(Boolean);
  let targetIds = [...new Set(localIds)];
  try {
    const runs = await api.get('/api/runs');
    targetIds = SageScheduler.selectStopRunIds(runs, {
      localIds,
      sessionId: state.session && state.session.id,
      partnerSessionIds,
      workflowIds,
    });
    if (!targetIds.length) {
      showToast(CUR_LANG === 'en' ? 'No running task was found' : '没有找到正在运行的任务');
      return;
    }
    state.stopRequested = true;
    const responses = await Promise.allSettled(
      targetIds.map((runId) => api.post('/api/stop', { run_id: runId }))
    );
    const accepted = responses.filter(
      (result) => result.status === 'fulfilled' && result.value && result.value.ok
    ).length;
    if (state.abort) state.abort.abort();
    const deadline = Date.now() + 7000;
    let stillRunning = targetIds;
    while (Date.now() < deadline && stillRunning.length) {
      await new Promise((resolve) => setTimeout(resolve, 120));
      const latest = await api.get('/api/runs');
      const active = new Set(latest.filter((run) => run.running).map((run) => run.run_id));
      stillRunning = targetIds.filter((runId) => active.has(runId));
    }
    if (stillRunning.length) {
      showToast(
        CUR_LANG === 'en'
          ? `Stop timed out for ${stillRunning.length} task(s)`
          : `仍有 ${stillRunning.length} 个任务未停止，请重试`
      );
    } else {
      showToast(
        CUR_LANG === 'en'
          ? `Stopped ${accepted || targetIds.length} task(s)`
          : `已停止 ${accepted || targetIds.length} 个任务`
      );
    }
    await pollRuns();
    loadConvs();
  } catch (error) {
    if (state.abort) state.abort.abort();
    showToast((CUR_LANG === 'en' ? 'Stop failed: ' : '停止失败：') + (error.message || error));
  } finally {
    state.stopping = false;
    if (!state.streaming) state.stopRequested = false;
    setSendButton(state.streaming);
  }
}

/** 仅断开查看连接，后台任务继续运行 */
function detachViewer() {
  if (state.abort) state.abort.abort();
}

function onNewSession() {
  if (state.streaming) detachViewer(); // 任务转后台继续，不中断
  showHero();
  loadAgentPrefs(state.agent); // 自动路由的模型只绑定该会话，新会话恢复手动偏好
  syncAgentUI();
}

function setChatHead(sess) {
  setBadge($('#chat-agent-badge'), sess.agent);
  $('#chat-title').textContent = sess.title || t('(无标题)');
  const proj = $('#chat-project');
  proj.textContent = sess.project || '';
  proj.title = sess.project || '';
}

/* ---------- 转录渲染 ---------- */

/** 协作注入消息（分工任务书/复查意见/汇总回注）：正文冗长，渲染为默认收起的卡片 */
const COLLAB_PREFIXES = [
  ['【SAGE COLLABORATE', '🤝 协作分工任务书'],
  ['【协作分工】', '🤝 协作分工任务书'],
  ['【协作汇总】', '🤝 分工产出回注'],
  ['【协作复查回注】', '🤝 复查意见回注'],
  ['【协作复查】', '🤝 协作复查任务书'],
  ['【协作追问】', '🤝 追问分派任务书'],
];

function collabInjectCard(text) {
  const hit = COLLAB_PREFIXES.find(([p]) => text.startsWith(p));
  if (!hit) return null;
  const card = el('div', 'card collab-inject');
  const head = el('div', 'card-head');
  head.appendChild(el('span', 'card-caret', '▸'));
  head.appendChild(el('span', 'card-title', t(hit[1])));
  const body = el('div', 'card-body');
  const pre = el('pre', 'io-pre');
  pre.textContent = text;
  body.appendChild(pre);
  card.appendChild(head);
  card.appendChild(body);
  head.addEventListener('click', () => card.classList.toggle('open'));
  return card;
}

function appendUserBubble(container, text, imgs) {
  // 协作流程的注入消息不占满屏——折叠卡展示，点击可看全文
  if (text && !(imgs && imgs.length)) {
    const cc = collabInjectCard(text);
    if (cc) {
      container.appendChild(cc);
      return;
    }
  }
  const wrap = el('div', 'msg-user');
  if (imgs && imgs.length) {
    const strip = el('div', 'msg-img-strip');
    for (const im of imgs) strip.appendChild(im);
    wrap.appendChild(strip);
  }
  if (text) wrap.appendChild(el('div', 'bubble', text));
  if (wrap.childElementCount) container.appendChild(wrap);
}

/** 图片渲染：data URL 直显；本地绝对路径经 /api/file；其余占位 */
function imageEl(src) {
  let url = null;
  if (/^data:image\//.test(src)) url = src;
  else if (/^([a-zA-Z]:[\\/]|\/|\\\\)/.test(src)) url = '/api/file?path=' + encodeURIComponent(src);
  if (!url) return el('div', 'md-text img-ph', '[图片] ' + snippet(src, 60));
  const img = el('img', 'msg-img');
  img.src = url;
  img.loading = 'lazy';
  img.alt = '图片';
  img.title = t('点击放大');
  img.addEventListener('click', () => openLightbox(url));
  img.addEventListener('error', () => {
    img.replaceWith(el('div', 'md-text img-ph', '[图片加载失败] ' + snippet(src, 80)));
  });
  return img;
}

/* ---------- 图片灯箱（滚轮缩放 / 拖拽平移 / 双击复位 / Esc 关闭） ---------- */

let lightboxEl = null;

function closeLightbox() {
  if (lightboxEl) {
    lightboxEl.remove();
    lightboxEl = null;
  }
}

function openLightbox(url) {
  closeLightbox();
  const ov = el('div', 'lightbox');
  const img = el('img', 'lightbox-img');
  img.src = url;
  img.alt = '图片';
  let scale = 1;
  let tx = 0;
  let ty = 0;
  const apply = () => {
    img.style.transform = 'translate(' + tx + 'px,' + ty + 'px) scale(' + scale + ')';
  };
  ov.appendChild(img);

  const bar = el('div', 'lightbox-bar');
  const openTab = el('a', 'lightbox-btn', t('⧉ 新标签打开'));
  openTab.href = url;
  openTab.target = '_blank';
  openTab.rel = 'noopener';
  openTab.addEventListener('click', (e) => e.stopPropagation());
  const closeBtn = el('button', 'lightbox-btn', t('✕ 关闭'));
  closeBtn.type = 'button';
  closeBtn.addEventListener('click', closeLightbox);
  bar.appendChild(openTab);
  bar.appendChild(closeBtn);
  ov.appendChild(bar);

  ov.addEventListener('click', (e) => {
    if (e.target === ov) closeLightbox();
  });
  ov.addEventListener(
    'wheel',
    (e) => {
      e.preventDefault();
      scale = Math.min(6, Math.max(0.2, scale * (e.deltaY < 0 ? 1.15 : 1 / 1.15)));
      apply();
    },
    { passive: false }
  );
  let drag = null;
  img.addEventListener('mousedown', (e) => {
    e.preventDefault();
    drag = { x: e.clientX - tx, y: e.clientY - ty };
    img.classList.add('dragging');
  });
  ov.addEventListener('mousemove', (e) => {
    if (drag) {
      tx = e.clientX - drag.x;
      ty = e.clientY - drag.y;
      apply();
    }
  });
  ov.addEventListener('mouseup', () => {
    drag = null;
    img.classList.remove('dragging');
  });
  img.addEventListener('dblclick', () => {
    scale = 1;
    tx = 0;
    ty = 0;
    apply();
  });
  img.addEventListener('click', (e) => e.stopPropagation());

  document.body.appendChild(ov);
  lightboxEl = ov;
}

/** 文本中 [Image: source: 路径] 模式 → 图片元素（Claude 引用的 image-cache 截图等） */
function extractInlineImages(text) {
  const out = [];
  const re = /\[Image:\s*source:\s*([^\]\n]+?)\s*\]/g;
  let m;
  while ((m = re.exec(text || ''))) out.push(imageEl(m[1].trim()));
  return out;
}

function renderDivider(text) {
  const d = el('div', 'divider');
  // 报错分隔线标红（转录里的 ⚠ 运行报错：…）
  if (text && text.startsWith('⚠ 运行报错')) d.classList.add('divider-error');
  d.appendChild(el('span', null, text || '· · ·'));
  return d;
}

function thinkingCard(text, open) {
  const card = el('div', 'card thinking' + (open ? ' open' : ''));
  const head = el('div', 'card-head');
  head.appendChild(el('span', 'card-caret', '▸'));
  head.appendChild(el('span', 'card-title', t('💭 思考过程')));
  const body = el('div', 'card-body think-body', text || '');
  card.appendChild(head);
  card.appendChild(body);
  head.addEventListener('click', () => card.classList.toggle('open'));
  return card;
}

/** 团队模式：本应用无法直接调用 agent 的派生工具，只能把组队要求写进 prompt。
 *  claude 用 Agent + SendMessage（队员间可互通），codex 用 spawn_agent（星型汇总）。 */
function withTeamPreamble(prompt, agent) {
  if (!state.teamOn || !prompt.trim()) return prompt;
  const n = state.teamSize;
  const head =
    agent === 'codex'
      ? `请用 spawn_agent 派生 ${n} 个子 agent 并行完成下面的任务：把任务拆成互相独立的部分分给它们，` +
        `用 wait 收齐结果后由你汇总。若任务不适合拆分，直接自己做，不要强行派生。`
      : `请用 Agent 工具启动 ${n} 个 general-purpose 子 agent 组成团队完成下面的任务：给每个队员起名并分配独立的一部分，` +
        `队员之间需要交换信息时用 SendMessage 互相沟通，最后由你汇总。若任务不适合拆分，直接自己做，不要强行组队。`;
  return head + '\n\n任务：\n' + prompt;
}

/** 团队协作类工具的可读摘要：Agent 显示队员名，SendMessage 显示投递对象。
 *  其余工具沿用参数首行。 */
function toolHeadInfo(name, inputText) {
  let inp = null;
  try {
    inp = JSON.parse(inputText);
  } catch (_) { /* 非 JSON 参数：按原样处理 */ }
  if (inp && typeof inp === 'object') {
    if (name === 'Agent' && (inp.name || inp.description)) {
      return { name: 'Agent · ' + (inp.name || inp.subagent_type || ''), summary: oneLine(inp.description || '') };
    }
    if (name === 'SendMessage' && (inp.to || inp.recipient)) {
      return { name: '→ ' + (inp.to || inp.recipient), summary: oneLine(inp.summary || inp.message || '') };
    }
  }
  return { name, summary: oneLine(inputText) };
}

function toolCard(name, inputText, id) {
  const card = el('div', 'card tool');
  if (id) card.dataset.tuid = id; // 子 agent 事件按调用 id 归入本卡片
  const head = el('div', 'card-head');
  head.appendChild(el('span', 'card-caret', '▸'));
  head.appendChild(el('span', 'tool-icon', '⚙'));
  const info = toolHeadInfo(name, inputText);
  head.appendChild(el('span', 'tool-name', info.name));
  head.appendChild(el('span', 'tool-summary', info.summary));
  const body = el('div', 'card-body');
  if (inputText) {
    body.appendChild(el('div', 'io-label', '输入'));
    const pre = el('pre', 'io-pre');
    pre.textContent = inputText;
    body.appendChild(pre);
  }
  card.appendChild(head);
  card.appendChild(body);
  head.addEventListener('click', () => card.classList.toggle('open'));
  return card;
}

/** 子 agent（Task 工具）过程：嵌进父工具卡片，默认随卡片折叠。 */
function appendSubTo(card, kind, name, text) {
  const body = card && card.querySelector('.card-body');
  if (!body) return;
  let sub = card.querySelector('.sub-run');
  if (!sub) {
    sub = el('div', 'sub-run');
    sub.appendChild(el('div', 'io-label', t('子 agent 过程')));
    body.appendChild(sub);
    const head = card.querySelector('.card-head');
    if (head && !head.querySelector('.sub-badge')) {
      head.insertBefore(el('span', 'sub-badge', '🤖'), head.querySelector('.tool-summary'));
    }
  }
  if (kind === 'text') {
    const d = el('div', 'sub-text');
    d.textContent = text;
    sub.appendChild(d);
  } else {
    const d = el('div', 'sub-tool');
    d.appendChild(el('span', 'sub-tool-name', name));
    d.appendChild(el('span', 'sub-tool-arg', oneLine(text)));
    sub.appendChild(d);
  }
}

/** 实时流：按父 tool_use id 找卡片（嵌套子 agent 的父卡片不在视图时静默丢弃）。 */
function appendSubEvent(parentId, kind, name, text) {
  if (!parentId) return;
  appendSubTo(chatMsgs.querySelector('[data-tuid="' + CSS.escape(parentId) + '"]'), kind, name, text);
}

/* ---------- 工具聚合分组（参考 Codex Desktop：一行小折叠，点开看细节） ---------- */

const TOOL_EDIT_RE = /edit|write|patch|apply|notebook/i;
const TOOL_READ_RE = /^(read|grep|glob|search|cat|ls)$/i;

/** ctx: {bodyEl, lastTool, lastGroup, planCard} — 一条助手消息的渲染上下文 */
function ensureToolGroup(ctx) {
  if (ctx.lastGroup) return ctx.lastGroup;
  const root = el('div', 'tgroup');
  const head = el('div', 'tgroup-head');
  const ico = el('span', 'tgroup-ico', '⊡');
  head.appendChild(ico);
  const label = el('span', 'tgroup-label', t('运行了命令'));
  head.appendChild(label);
  const body = el('div', 'tgroup-body');
  root.appendChild(head);
  root.appendChild(body);
  head.addEventListener('click', () => root.classList.toggle('open'));
  ctx.bodyEl.appendChild(root);
  ctx.lastGroup = { root, body, label, ico, count: 0, edit: false, run: false, read: false };
  return ctx.lastGroup;
}

/** 参考 Codex Desktop：类别名直接连写，如「编辑了文件读取了文件运行了命令」 */
function groupLabel(g) {
  const parts = [];
  if (g.edit) parts.push(t('编辑了文件'));
  if (g.read) parts.push(t('读取了文件'));
  if (g.run) parts.push(t('运行了命令'));
  return parts.length ? parts.join(CUR_LANG === 'en' ? ' · ' : '') : t('执行了操作');
}

/** 文本/思考/分隔出现时结束当前分组（后续工具开新组） */
function endToolGroup(ctx) {
  if (ctx) ctx.lastGroup = null;
}

function appendToolUse(ctx, name, text, id) {
  const g = ensureToolGroup(ctx);
  g.count++;
  if (TOOL_EDIT_RE.test(name)) g.edit = true;
  else if (TOOL_READ_RE.test(name)) g.read = true;
  else g.run = true;
  g.label.textContent = groupLabel(g);
  g.ico.textContent = g.edit ? '✎' : g.run ? '⊡' : '☰';
  const card = toolCard(name, text, id);
  g.body.appendChild(card);
  ctx.lastTool = card;
  return card;
}

function appendToolResult(ctx, text) {
  if (!ctx.lastTool) appendToolUse(ctx, '结果', '');
  const body = ctx.lastTool.querySelector('.card-body');
  body.appendChild(el('div', 'io-label', '输出'));
  const pre = el('pre', 'io-pre');
  pre.textContent = text || '(空)';
  body.appendChild(pre);
}

/* ---------- 已编辑文件卡片（默认 3 个 + 展开 / 点击差异 / 右键菜单） ---------- */

function flushFilesCard(ctx) {
  if (!ctx || !ctx.fileEdits || !ctx.fileEdits.size) return;
  const files = [...ctx.fileEdits];
  ctx.fileEdits.clear();
  const proj = state.session ? state.session.project : state.project;
  ctx.bodyEl.appendChild(filesCard(files, proj));
}

function filesCard(files, project) {
  const card = el('div', 'files-card');
  const head = el('div', 'files-head');
  head.appendChild(el('span', 'files-ico-box', '🗂'));
  const col = el('div', 'files-head-col');
  col.appendChild(el('div', 'files-title', tEditedFiles(files.length)));
  const totals = el('div', 'files-totals');
  col.appendChild(totals);
  head.appendChild(col);
  card.appendChild(head);
  const list = el('div', 'files-list');
  card.appendChild(list);
  const LIMIT = 3;
  const rows = files.map((f) => fileRow(f, project));
  rows.forEach((r, i) => {
    if (i >= LIMIT) r.classList.add('hidden');
    list.appendChild(r);
  });
  if (rows.length > LIMIT) {
    const more = el('button', 'files-more', tMoreFiles(rows.length - LIMIT));
    more.type = 'button';
    let open = false;
    more.addEventListener('click', () => {
      open = !open;
      rows.forEach((r, i) => {
        if (i >= LIMIT) r.classList.toggle('hidden', !open);
      });
      more.textContent = open ? t('收起文件 ⌃') : tMoreFiles(rows.length - LIMIT);
    });
    card.appendChild(more);
  }
  // 异步补 +/- 统计（git numstat）
  api
    .post('/api/filestat', { project: project || '', files })
    .then((stats) => {
      let ta = 0;
      let td = 0;
      let has = false;
      (stats || []).forEach((s2, i) => {
        if (s2 && s2.adds !== null && s2.adds !== undefined) {
          has = true;
          ta += s2.adds;
          td += s2.dels || 0;
          const cell = rows[i] && rows[i].querySelector('.files-diffstat');
          if (cell) {
            cell.appendChild(el('span', 'stat-add', '+' + s2.adds));
            cell.appendChild(el('span', 'stat-del', '-' + (s2.dels || 0)));
          }
        }
      });
      if (has) {
        totals.appendChild(el('span', 'stat-add', '+' + ta));
        totals.appendChild(el('span', 'stat-del', '-' + td));
      }
    })
    .catch(() => {});
  return card;
}

function fileRow(f, project) {
  const row = el('div', 'files-row');
  let norm = f.replace(/\\/g, '/');
  // 项目内文件显示相对路径（参考 Codex Desktop）
  const projNorm = (project || '').replace(/\\/g, '/').replace(/\/+$/, '') + '/';
  if (projNorm.length > 1 && norm.toLowerCase().startsWith(projNorm.toLowerCase())) {
    norm = norm.slice(projNorm.length);
  }
  const idx = norm.lastIndexOf('/');
  const pathEl = el('span', 'files-path');
  if (idx > 0) pathEl.appendChild(el('span', 'files-dir', norm.slice(0, idx + 1)));
  pathEl.appendChild(el('span', 'files-name', norm.slice(idx + 1)));
  row.appendChild(pathEl);
  row.appendChild(el('span', 'files-diffstat'));
  row.title = f + '\n' + t('点击查看差异 · 右键更多操作');
  row.addEventListener('click', () => openDiffModal(project, f));
  row.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    e.stopPropagation();
    showFileMenu(e, f, project);
  });
  return row;
}

/** 差异审查视图：GitHub 风格 —— 行号双栏 + 整行着色 + 隐藏元信息噪音 */
async function openDiffModal(project, file) {
  closeLightbox();
  const ov = el('div', 'lightbox');
  const panel = el('div', 'diff-panel');
  const head = el('div', 'diff-head');
  // 文件名突出：目录灰色 + 文件名加粗
  const norm = file.replace(/\\/g, '/');
  const li = norm.lastIndexOf('/');
  const title = el('span', 'diff-title');
  if (li > 0) title.appendChild(el('span', 'diff-dir', norm.slice(0, li + 1)));
  title.appendChild(el('span', 'diff-name', norm.slice(li + 1)));
  head.appendChild(title);
  const stats = el('span', 'diff-stats');
  head.appendChild(stats);
  const srcTag = el('span', 'diff-src');
  head.appendChild(srcTag);
  const x = el('button', 'diff-close', '✕');
  x.type = 'button';
  x.title = '关闭（Esc）';
  x.addEventListener('click', closeLightbox);
  head.appendChild(x);
  panel.appendChild(head);
  const body = el('div', 'diff-body');
  body.appendChild(el('div', 'empty', t('加载差异中…')));
  panel.appendChild(body);
  ov.appendChild(panel);
  ov.addEventListener('click', (e) => {
    if (e.target === ov) closeLightbox();
  });
  document.body.appendChild(ov);
  lightboxEl = ov;

  const row = (cls, oldN, newN, text) => {
    const r = el('div', 'diff-line ' + cls);
    r.appendChild(el('span', 'diff-ln', oldN === null ? '' : String(oldN)));
    r.appendChild(el('span', 'diff-ln', newN === null ? '' : String(newN)));
    r.appendChild(el('span', 'diff-code', text));
    return r;
  };

  try {
    const d = await api.get('/api/diff?' + new URLSearchParams({ project: project || '', file }));
    body.textContent = '';
    if (d && d.source) srcTag.textContent = d.source;
    const txt = (d && d.diff) || '';
    if (!txt.trim()) {
      body.appendChild(el('div', 'empty', t('没有可显示的差异（可能已提交，或与 HEAD 一致）')));
      return;
    }
    let oldLn = 1;
    let newLn = 1;
    let adds = 0;
    let dels = 0;
    for (const line of txt.split('\n')) {
      // 元信息行（diff --git / index / --- / +++）隐藏，减少噪音
      if (
        line.startsWith('diff ') ||
        line.startsWith('index ') ||
        line.startsWith('--- ') ||
        line.startsWith('+++ ') ||
        line.startsWith('new file') ||
        line.startsWith('deleted file')
      ) {
        continue;
      }
      const hm = line.match(/^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@(.*)$/);
      if (hm) {
        oldLn = parseInt(hm[1], 10);
        newLn = parseInt(hm[2], 10);
        const band = el('div', 'diff-hunk-band');
        band.textContent = '⋯' + (hm[3] ? '  ' + hm[3].trim() : '');
        body.appendChild(band);
        continue;
      }
      if (line.startsWith('+')) {
        adds++;
        body.appendChild(row('add', null, newLn++, line));
      } else if (line.startsWith('-')) {
        dels++;
        body.appendChild(row('del', oldLn++, null, line));
      } else if (line.length || body.lastElementChild) {
        body.appendChild(row('ctx', oldLn++, newLn++, line.length ? line : ' '));
      }
    }
    stats.appendChild(el('span', 'stat-add', '+' + adds));
    stats.appendChild(el('span', 'stat-del', '-' + dels));
  } catch (e) {
    body.textContent = '';
    body.appendChild(el('div', 'empty', t('差异加载失败：') + e.message));
  }
}

/** 文件行右键菜单：VS Code / 资源管理器 / 复制路径 / 复制内容 */
function showFileMenu(e, file, project) {
  closeMenu();
  const menu = el('div', 'menu');
  menu.addEventListener('click', (ev) => ev.stopPropagation());
  const items = [
    [t('在 VS Code 中打开'), () =>
      api.post('/api/open', { path: file, project }).catch((er) => alert(t('打开失败：') + er.message))],
    [t('在资源管理器中打开'), () =>
      api
        .post('/api/open', { path: file, project, mode: 'reveal' })
        .catch((er) => alert(t('打开失败：') + er.message))],
    [t('复制路径'), () => navigator.clipboard.writeText(file).catch(() => {})],
    [t('复制文件内容'), async () => {
      try {
        const d = await api.get(
          '/api/filetext?' + new URLSearchParams({ path: file, project: project || '' })
        );
        await navigator.clipboard.writeText(d.text || '');
      } catch (er) {
        alert(t('复制失败：') + er.message);
      }
    }],
  ];
  for (const [label, fn] of items) {
    const btn = el('button', 'menu-item', label);
    btn.type = 'button';
    btn.addEventListener('click', () => {
      closeMenu();
      fn();
    });
    menu.appendChild(btn);
  }
  document.body.appendChild(menu);
  const mw = menu.offsetWidth;
  const mh = menu.offsetHeight;
  menu.style.left = Math.max(8, Math.min(e.clientX, window.innerWidth - mw - 8)) + 'px';
  menu.style.top = Math.max(8, Math.min(e.clientY, window.innerHeight - mh - 8)) + 'px';
  menuEl = menu;
  menuOwner = null;
}

/* ---------- 任务计划卡片（TodoWrite / update_plan 进度清单） ---------- */

function planIcon(status) {
  if (status === 'completed') return '✓';
  if (status === 'in_progress') return '●';
  return '○';
}

function renderPlanInto(card, items) {
  card.textContent = '';
  const doneN = items.filter((i) => i.status === 'completed').length;
  const head = el('div', 'plan-head');
  head.appendChild(el('span', 'plan-title', t('📋 任务计划')));
  head.appendChild(el('span', 'plan-progress', doneN + ' / ' + items.length));
  card.appendChild(head);
  const box = el('div', 'plan-steps');
  for (const it of items) {
    const row = el('div', 'plan-step ' + (it.status || 'pending'));
    row.appendChild(el('span', 'plan-ico', planIcon(it.status)));
    row.appendChild(el('span', 'plan-text', it.text || ''));
    box.appendChild(row);
  }
  card.appendChild(box);
  head.addEventListener('click', () => card.classList.toggle('folded'));
}

/** 同一条助手消息内的计划更新复用同一张卡片（只展示最新状态） */
function upsertPlan(ctx, items) {
  if (!items || !items.length) return;
  if (!ctx.planCard) {
    ctx.planCard = el('div', 'plan-card');
    ctx.bodyEl.appendChild(ctx.planCard);
  }
  renderPlanInto(ctx.planCard, items);
}

function renderAssistantMsg(container, blocks, pos, withActions) {
  const bodyEl = el('div', 'msg-asst');
  container.appendChild(bodyEl);
  const ctx = { bodyEl, lastTool: null, lastGroup: null, planCard: null, fileEdits: new Set() };
  for (const b of blocks) {
    if (b.kind === 'file_edit') {
      ctx.fileEdits.add(b.text);
    } else if (b.kind === 'text') {
      endToolGroup(ctx);
      const d = el('div', 'md');
      renderMarkdown(d, b.text);
      bodyEl.appendChild(d);
    } else if (b.kind === 'thinking') {
      endToolGroup(ctx);
      bodyEl.appendChild(thinkingCard(b.text, false));
    } else if (b.kind === 'tool_use') {
      appendToolUse(ctx, b.name || '工具', b.text);
    } else if (b.kind === 'sub_text' || b.kind === 'sub_tool') {
      // 历史回放：紧随其父 tool_use 块，归入刚建好的那张卡片
      if (ctx.lastTool) {
        appendSubTo(ctx.lastTool, b.kind === 'sub_text' ? 'text' : 'tool', b.name || '工具', b.text);
      }
    } else if (b.kind === 'tool_result') {
      appendToolResult(ctx, b.text);
    } else if (b.kind === 'plan') {
      let items = [];
      try {
        items = JSON.parse(b.text) || [];
      } catch (_) { /* 忽略坏数据 */ }
      upsertPlan(ctx, items);
    } else if (b.kind === 'divider') {
      endToolGroup(ctx);
      bodyEl.appendChild(renderDivider(b.text));
    } else if (b.kind === 'image') {
      endToolGroup(ctx);
      bodyEl.appendChild(imageEl(b.text));
    }
  }
  const txt = blocks
    .filter((b) => b.kind === 'text' && b.text)
    .map((b) => b.text)
    .join('\n');
  // 只有整轮回答的末条才挂操作栏：CLI 会把一次回答拆成许多条消息，
  // 每条都挂会让每段话下面都多出一行按钮。
  const bar = withActions ? appendMsgActions(bodyEl, txt) : null;
  // 有定位信息（行 ordinal/uuid）→ 支持真·从此消息处分叉
  if (bar && pos !== undefined && pos !== null) bar.appendChild(makeForkBtn(pos));
  return ctx;
}

/** 助手消息悬浮操作栏：每条消息可复制；⑂ 分支只挂在最后一条回复
 *（CLI 仅支持从会话当前末尾分叉，挂历史消息会误导「从该处分叉」）。 */
function appendMsgActions(bodyEl, text) {
  if (!text || !text.trim()) return null; // 无正文的消息不放操作栏（避免悬空按钮）
  const bar = el('div', 'msg-actions');
  const copyBtn = el('button', 'msg-act', '⧉');
  copyBtn.type = 'button';
  copyBtn.title = t('复制正文');
  copyBtn.addEventListener('click', () => {
    navigator.clipboard
      .writeText(text || '')
      .then(() => showToast(t('已复制')))
      .catch(() => showToast(t('复制失败')));
  });
  bar.appendChild(copyBtn);
  bodyEl.appendChild(bar);
  return bar;
}

/** 分叉：pos=null 从会话末尾；否则从指定消息处（codex=行 ordinal，claude=行 uuid）。
 *  由后端合成分叉文件（codex 引用式 / claude 复制截断），零模型调用即时完成。 */
async function forkAt(pos) {
  if (!state.session || !state.session.id) {
    showToast(t('会话尚未保存，无法分支'));
    return;
  }
  if (state.streaming) {
    showToast(CUR_LANG === 'en' ? 'A task is running — fork after it finishes' : '任务运行中，结束后再分支');
    return;
  }
  const s = state.session;
  try {
    const r = await api.post('/api/fork-at', {
      agent: s.agent,
      id: s.id,
      project: s.project,
      pos: pos === undefined ? null : pos,
    });
    // 分支 = 父会话当时的样子：协作子会话关联一并继承（归属仍在父会话）
    const st = collabStoreLoad();
    for (const ln of st.links[s.agent + ':' + s.id] || []) {
      collabLinkSave(s.agent + ':' + r.id, { ...ln }, ln.partner);
    }
    showToast(CUR_LANG === 'en' ? '⑂ Forked — now in the new session' : '⑂ 已分支，当前已切到新会话');
    loadConvs();
    renderProjects();
    openSession({ agent: s.agent, id: r.id, project: s.project, title: '' });
  } catch (e) {
    alert(t('分支失败：') + e.message);
  }
}

function doForkNow() {
  forkAt(null);
}

function makeForkBtn(pos) {
  const forkBtn = el('button', 'msg-act', '⑂');
  forkBtn.type = 'button';
  forkBtn.title =
    pos !== null && pos !== undefined
      ? t('从此消息处分叉出新会话（之后的内容不带入，原会话不受影响）')
      : t('分支到新会话：基于当前会话状态分叉，原会话不受影响');
  forkBtn.addEventListener('click', () => forkAt(pos === undefined ? null : pos));
  return forkBtn;
}

/** 返回新的 lastAsst（用户真实输入会开启新回合 → null） */
/** 发图的文本标记；正文展示时剥离，故比对气泡内容前也要剥一次 */
const IMG_REF_SRC = '请查看图片文件[:：]\\s*([^\\s，。;；\\n]+\\.(?:png|jpe?g|gif|webp|bmp))';

/** 一段 prompt 在气泡里实际显示的正文（剥掉图片标记后） */
function visibleUserText(s) {
  return SageScheduler.visibleUserPrompt(s)
    .replace(new RegExp(IMG_REF_SRC, 'gi'), '')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
}

function renderUserMsg(container, blocks, lastAsst) {
  const toolResults = blocks.filter((b) => b.kind === 'tool_result');
  const others = blocks.filter((b) => b.kind !== 'tool_result');
  if (toolResults.length) {
    // tool_result-only 的 user 行并入上一条助手消息展示
    if (!lastAsst) {
      const bodyEl = el('div', 'msg-asst');
      container.appendChild(bodyEl);
      lastAsst = { bodyEl, lastTool: null, lastGroup: null, planCard: null };
    }
    for (const b of toolResults) appendToolResult(lastAsst, b.text);
  }
  const textParts = [];
  const imgs = [];
  for (const b of others) {
    if (b.kind === 'image') imgs.push(imageEl(b.text));
    else if (b.kind === 'divider') container.appendChild(renderDivider(b.text));
    else if (b.text) textParts.push(b.text);
  }
  let joined = textParts.join('\n').replace(/\s+$/, '');
  imgs.push(...extractInlineImages(joined));
  // 「请查看图片文件: <路径>」（本应用发图的文本形式）→ 还原为缩略图并从正文剥离
  const IMG_REF_RE = new RegExp(IMG_REF_SRC, 'gi');
  let mm;
  while ((mm = IMG_REF_RE.exec(joined))) imgs.push(imageEl(mm[1]));
  joined = joined
    .replace(IMG_REF_RE, '')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
  if (joined || imgs.length) {
    flushFilesCard(lastAsst); // 用户真实输入前，结清上一轮的文件卡片
    appendUserBubble(container, joined, imgs);
    return null;
  }
  return lastAsst;
}

/** 每轮回答的末条助手消息下标集合：CLI 把一次回答拆成多条，操作栏只挂末条。
 *  轮次以「真实用户输入」（含 tool_result 之外的块）为界。 */
function turnFinalAssistants(msgs) {
  const out = new Set();
  let cand = -1;
  msgs.forEach((m, i) => {
    const blocks = m.blocks || [];
    if (m.role === 'assistant' && blocks.some((b) => b.kind === 'text' && b.text)) cand = i;
    else if (m.role === 'user' && blocks.some((b) => b.kind !== 'tool_result') && cand >= 0) {
      out.add(cand);
      cand = -1;
    }
  });
  if (cand >= 0) out.add(cand);
  return out;
}

function renderTranscript(container, tr) {
  // 参数不可叫 t：会遮蔽全局翻译函数 t()，空消息分支曾因此抛 "t is not a function"
  if (!tr.messages || !tr.messages.length) {
    container.appendChild(el('div', 'empty center', t('（此会话没有可显示的消息）')));
    return;
  }
  const finals = turnFinalAssistants(tr.messages);
  let lastAsst = null;
  for (let mi = 0; mi < tr.messages.length; mi++) {
    const m = tr.messages[mi];
    if (m.role === 'assistant') {
      const blocks = m.blocks || [];
      // 纯工具消息并入上一条助手消息的分组（跨消息聚合，避免一串「运行了命令」）
      const onlyTools =
        blocks.length > 0 &&
        blocks.every(
          (b) => b.kind === 'tool_use' || b.kind === 'tool_result' || b.kind === 'file_edit'
        );
      if (onlyTools && lastAsst) {
        for (const b of blocks) {
          if (b.kind === 'tool_use') appendToolUse(lastAsst, b.name || '工具', b.text);
          else if (b.kind === 'file_edit') lastAsst.fileEdits.add(b.text);
          else appendToolResult(lastAsst, b.text);
        }
      } else {
        // 文件改动跨消息累计，整轮只在末尾出一张汇总卡
        const carry = lastAsst ? lastAsst.fileEdits : null;
        lastAsst = renderAssistantMsg(container, blocks, m.pos, finals.has(mi));
        if (carry && carry.size) carry.forEach((p) => lastAsst.fileEdits.add(p));
      }
    } else if (m.role === 'user') {
      lastAsst = renderUserMsg(container, m.blocks || [], lastAsst);
    } else {
      for (const b of m.blocks || []) container.appendChild(renderDivider(b.text));
    }
  }
  flushFilesCard(lastAsst);
}

/** 重连后台运行：只跟新事件，结束后重载权威转录补齐 */
async function attachRun(runId, sess) {
  state.runId = runId;
  state.streaming = true;
  setSendButton(true);
  beginAssistant();
  stream.ctx.bodyEl.appendChild(renderDivider(t('页面重连 · 以下为实时输出')));
  stream.ctx.bodyEl.appendChild(cursorEl);
  const ac = new AbortController();
  state.abort = ac;
  let aborted = false;
  try {
    const resp = await fetch('/api/run?' + new URLSearchParams({ id: runId }), {
      signal: ac.signal,
    });
    if (resp.ok && resp.body) await readNdjson(resp, handleEvent);
  } catch (e) {
    aborted = !!(e && e.name === 'AbortError');
    if (aborted && state.stopRequested && stream) {
      stream.ctx.bodyEl.appendChild(el('div', 'status-line', t('■ 已停止')));
    }
  }
  finalizeStream();
  state.streaming = false;
  state.abort = null;
  state.runId = null;
  state.runIds.clear();
  setSendButton(false);
  if (state.stopRequested) state.stopRequested = false;
  // 任务结束且仍停留在该会话：重载完整转录（补齐重连前缺失的部分）
  if (!aborted && sess && state.session && state.session.id === sess.id) {
    openSession(state.session);
    loadConvs();
  }
}

/** 打开会话后检查是否有其后台运行，有则接上 */
async function maybeAttachSessionRun(s) {
  if (state.streaming) return;
  try {
    const runs = await api.get('/api/runs');
    const r = runs.find((x) => x.running && x.session_id === s.id);
    if (r && state.session && state.session.id === s.id) attachRun(r.run_id, s);
  } catch (_) {
    /* 忽略 */
  }
}

function syncBackPrimaryUI(s) {
  const key = s && s.id ? s.agent + ':' + s.id : '';
  const store = collabStoreLoad();
  state.backPrimary = key ? store.back[key] || null : null;
  const backEntry = state.backPrimary
    ? (store.links[state.backPrimary] || []).find((entry) => entry.partner === key)
    : null;
  const handoffBack = backEntry && backEntry.kind === 'handoff';
  const bb = $('#back-primary-btn');
  const fb = $('#feed-primary-btn');
  if (bb) {
    bb.classList.toggle('hidden', !state.backPrimary);
    bb.textContent = handoffBack
      ? (CUR_LANG === 'en' ? '← Source session' : '← 来源会话')
      : (CUR_LANG === 'en' ? '← Main session' : '← 主会话');
  }
  if (fb) {
    fb.classList.toggle('hidden', !state.backPrimary || handoffBack);
    fb.textContent = CUR_LANG === 'en' ? '⇪ Feed back' : '⇪ 回注主会话';
    fb.title =
      CUR_LANG === 'en'
        ? 'Send this sub-session\'s latest conclusion to the main session to act on'
        : '把子会话最新结论交给主会话消化落实（主会话不会自动看见子会话内容）';
  }
}

async function openSession(s) {
  if (state.streaming) detachViewer(); // 任务转后台继续，不中断
  const savedDecision = s.id ? sageStoreLoad()[s.agent + ':' + s.id] || null : null;
  const savedExecutor = savedDecision ? sageExecutor(savedDecision, savedDecision.primary) : null;
  const sessionModel =
    s.model || (savedExecutor && savedExecutor.runtime === s.agent ? savedExecutor.model : null);
  const sessionEffort = s.effort || savedDecision && savedDecision.primary_effort || null;
  state.session = {
    agent: s.agent,
    id: s.id,
    project: s.project,
    title: s.title || t('(无标题)'),
    model: sessionModel || null,
    effort: sessionEffort,
  };
  setAgent(s.agent); // 权限/模型下拉选项联动到该会话的 agent
  if (sessionModel) {
    state.model = sessionModel;
    syncAgentUI();
  }
  if (sessionEffort) {
    state.effort = sessionEffort;
    syncAgentUI();
  }
  expandProjectFor(state.session); // 侧栏展开所属项目并定位
  setActiveRow(s.agent + ':' + s.id);
  showChat();
  setChatHead(state.session);
  renderCollabPanel(); // 右侧子会话面板（无关联则隐藏）
  // 子会话：头部常驻「返回主会话 / 回注主会话」（顶部横幅滚下去就看不见了）
  syncBackPrimaryUI(state.session);
  const ub = $('#usage-bar');
  if (ub) ub.classList.add('hidden'); // 历史会话无实时用量数据
  promptInput.placeholder = t('继续这个会话…');
  hideComposerError();
  chatMsgs.textContent = '';
  renderSkeleton(chatMsgs, 4, true);
  try {
    const qs = new URLSearchParams({ agent: s.agent, id: s.id, project: s.project || '' });
    const t = await api.get('/api/session?' + qs);
    if (!state.session || state.session.id !== s.id || state.session.agent !== s.agent) return; // 已切走
    // 会话自己录到的模型优先于界面当前选择：resume 会带模型下发，
    // 不回填就等于用侧栏上次选的模型顶掉这条会话原本在跑的模型。
    const recordedModel = t.model || (t.usage && t.usage.model);
    if (recordedModel && !sessionModel) {
      state.session.model = recordedModel;
      state.model = recordedModel;
      syncAgentUI();
    }
    chatMsgs.textContent = '';
    renderTranscript(chatMsgs, t);
    // 该会话若有路由决策记录 → 在首条回答前重现决策卡（默认折叠）
    const sd = sageStoreLoad()[s.agent + ':' + s.id];
    if (sd) {
      const card = sageCard(sd);
      card.classList.remove('open');
      const firstAsst = chatMsgs.querySelector('.msg-asst');
      if (firstAsst) chatMsgs.insertBefore(card, firstAsst);
      else chatMsgs.appendChild(card);
    }
    stitchCollab(s, t); // 协作子会话缝合（异步补入，不阻塞打开；旧协作按内容回溯配对）
    state.histUsage = t.usage || null; // 供续聊/重连作为用量基线
    if (t.usage) renderUsageFromHistory(t.usage, s.agent); // 已完成会话的整场用量
    if (t.title) {
      state.session.title = t.title;
      setChatHead(state.session);
    }
    scrollChat();
    maybeAttachSessionRun(s);
  } catch (e) {
    if (!state.session || state.session.id !== s.id || state.session.agent !== s.agent) return;
    chatMsgs.textContent = '';
    chatMsgs.appendChild(el('div', 'error-bar', t('转录加载失败：') + e.message));
    const retry = el('button', 'btn-ghost retry', t('重试'));
    retry.type = 'button';
    retry.addEventListener('click', () => openSession(s));
    chatMsgs.appendChild(retry);
    maybeAttachSessionRun(s); // 会话文件未落盘的运行中任务：转录取不到也接上实时流
  }
  promptInput.focus();
}

/* ---------- 滚动 ---------- */

function chatNearBottom() {
  return chatScrollEl.scrollHeight - chatScrollEl.scrollTop - chatScrollEl.clientHeight < 90;
}

function scrollChat() {
  chatScrollEl.scrollTop = chatScrollEl.scrollHeight;
}

/* ---------- NDJSON 流式对话 ---------- */

let stream = null; // {ctx:{bodyEl,lastTool}, cur, stderrPre}
let sageUsage = null; // SAGE 协作/移交期间跨子会话的用量聚合（形状对齐 stream）
const cursorEl = el('span', 'cursor');

function beginAssistant() {
  const bodyEl = el('div', 'msg-asst streaming');
  chatMsgs.appendChild(bodyEl);
  stream = {
    ctx: { bodyEl, lastTool: null, lastGroup: null, planCard: null, fileEdits: new Set() },
    cur: null,
    stderrPre: null,
    startedAt: Date.now(),
    usage: { input: 0, output: 0, cr: 0, cw: 0, ctx: 0, window: 0, has: false, abs: false },
    spd0: null, // 本轮速度基点（首个整场权威值的 {out, t}）
    finalText: '',
    doneOk: false,
    // 续聊/重连：以历史用量为基线，实时数值 = 基线 + 本轮增量
    base: state.histUsage
      ? {
          input: state.histUsage.input || 0,
          output: state.histUsage.output || 0,
          cr: state.histUsage.cache_read || 0,
          cw: state.histUsage.cache_write || 0,
          ctx: state.histUsage.context || 0,
          window: state.histUsage.window || 0,
          firstTs: state.histUsage.first_ts ? Date.parse(state.histUsage.first_ts) : null,
          model: state.histUsage.model || '',
        }
      : null,
    usageTimer: setInterval(renderUsageBar, 1000),
  };
  bodyEl.appendChild(cursorEl);
}

/* ---------- 用量条：总 token / 速度 / 缓存命中率 ---------- */

function fmtTok(n) {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M';
  if (n >= 1000) return (n / 1000).toFixed(1) + 'k';
  return String(n);
}

/** 上下文窗口大小：事件提供 > 所选模型查表 > 模型发现 > 按模型名推断 */
function contextWindowFor() {
  const u = stream && stream.usage;
  if (u && u.window) return u.window;
  const agent = currentAgent();
  const info = state.modelsInfo && state.modelsInfo[agent];
  const w = info && info.windows && info.windows[modelFull()];
  if (w) return w;
  if (info && info.context_window) return info.context_window;
  if (agent === 'claude') return /\[1m\]/i.test(modelFull()) ? 1000000 : 200000;
  return 0; // 未知则不显示占比
}

/** 历史会话的整场用量（后端从会话文件聚合） */
function renderUsageFromHistory(u, agent) {
  const bar = $('#usage-bar');
  if (!bar || !u) return;
  const input = u.input || 0;
  const out = u.output || 0;
  const cr = u.cache_read || 0;
  const cw = u.cache_write || 0;
  const durS =
    u.first_ts && u.last_ts
      ? Math.max(0, (Date.parse(u.last_ts) - Date.parse(u.first_ts)) / 1000)
      : 0;
  const speed = durS > 1 ? out / durS : 0;
  const den = input + cr + cw;
  const hit = den > 0 ? Math.round((cr / den) * 100) : 0;
  let text = '↑ ' + fmtTok(den) + ' · ↓ ' + fmtTok(out);
  if (speed > 0) text += ' · ' + speed.toFixed(1) + ' tok/s';
  text += ' · ' + t('缓存') + ' ' + hit + '%';
  let tip =
    'input ' + input + ' + cache_read ' + cr + ' + cache_write ' + cw + '\noutput ' + out;
  // 窗口：历史自带 > 会话模型查表 > 模型发现 > 按会话模型名推断
  let win = u.window || 0;
  if (!win) {
    const info = state.modelsInfo && state.modelsInfo[agent];
    if (info && info.windows && info.windows[u.model || '']) win = info.windows[u.model];
    else if (info && info.context_window) win = info.context_window;
    else if (agent === 'claude') win = /\[1m\]/i.test(u.model || '') ? 1000000 : 200000;
  }
  const ctx = u.context || 0;
  if (win > 0 && ctx > 0) {
    const used = Math.min(100, Math.round((ctx / win) * 100));
    text += ' · ' + t('上下文') + ' ' + used + '%';
    tip +=
      '\n' +
      (CUR_LANG === 'en'
        ? 'Context window: ' + used + '% used (' + (100 - used) + '% left)\n' +
          fmtTok(ctx) + ' of ' + fmtTok(win) + ' tokens'
        : '上下文窗口：' + used + '% 已用（剩余 ' + (100 - used) + '%）\n已用 ' +
          fmtTok(ctx) + '，共 ' + fmtTok(win));
  }
  bar.textContent = text;
  bar.title = tip;
  bar.classList.remove('hidden');
}

function renderUsageBar() {
  const bar = $('#usage-bar');
  // SAGE 协作/移交期间没有主流式对象，改用跨节点的用量聚合器
  const s = stream || sageUsage;
  if (!bar || !s) return;
  const b = s.base;
  if (!s.usage.has && !b) return;
  const raw = s.usage;
  // abs（scope=session 权威值）已含整场累计，直接用；否则基线（历史）+ 本轮增量
  const u = raw.abs
    ? {
        input: raw.input,
        output: raw.output,
        cr: raw.cr,
        cw: raw.cw,
        ctx: raw.ctx || (b ? b.ctx : 0),
        window: raw.window || (b ? b.window : 0),
      }
    : {
        input: raw.input + (b ? b.input : 0),
        output: raw.output + (b ? b.output : 0),
        cr: raw.cr + (b ? b.cr : 0),
        cw: raw.cw + (b ? b.cw : 0),
        ctx: raw.ctx || (b ? b.ctx : 0),
        window: raw.window || (b ? b.window : 0),
      };
  // 速度 = 本轮实时速率（Δ输出 / Δ时间），每秒随定时器刷新
  let speed;
  if (raw.abs && s.spd0) {
    speed =
      Math.max(0, raw.output - s.spd0.out) /
      Math.max(1, (Date.now() - s.spd0.t) / 1000);
  } else {
    speed = raw.output / Math.max(1, (Date.now() - s.startedAt) / 1000);
  }
  const den = u.input + u.cr + u.cw;
  const hit = den > 0 ? Math.round((u.cr / den) * 100) : 0;
  let text =
    '↑ ' + fmtTok(u.input + u.cr + u.cw) + ' · ↓ ' + fmtTok(u.output) +
    ' · ' + speed.toFixed(1) + ' tok/s · ' + t('缓存') + ' ' + hit + '%';
  let tip =
    'input ' + u.input + ' + cache_read ' + u.cr + ' + cache_write ' + u.cw +
    '\noutput ' + u.output;
  const win0 = u.window || contextWindowFor();
  const winEff =
    win0 > 0 ? win0 : s.base && /\[1m\]/i.test(s.base.model) ? 1000000 : 0;
  if (winEff > 0 && u.ctx > 0) {
    const used = Math.min(100, Math.round((u.ctx / winEff) * 100));
    text += ' · ' + t('上下文') + ' ' + used + '%';
    tip +=
      '\n' +
      (CUR_LANG === 'en'
        ? 'Context window: ' + used + '% used (' + (100 - used) + '% left)\n' +
          fmtTok(u.ctx) + ' of ' + fmtTok(winEff) + ' tokens'
        : '上下文窗口：' + used + '% 已用（剩余 ' + (100 - used) + '%）\n已用 ' +
          fmtTok(u.ctx) + '，共 ' + fmtTok(winEff));
  }
  bar.textContent = text;
  bar.title = tip;
  bar.classList.remove('hidden');
}

/** 耗时格式化：45 秒 / 3分15秒 / 1小时02分 */
function fmtDuration(ms) {
  const s = Math.round(ms / 1000);
  const en = CUR_LANG === 'en';
  if (s < 60) return en ? s + 's' : s + ' 秒';
  const m = Math.floor(s / 60);
  if (m < 60) return en ? m + 'm' + String(s % 60).padStart(2, '0') + 's' : m + '分' + String(s % 60).padStart(2, '0') + '秒';
  return en ? Math.floor(m / 60) + 'h' + String(m % 60).padStart(2, '0') + 'm' : Math.floor(m / 60) + '小时' + String(m % 60).padStart(2, '0') + '分';
}

function finalizeCur() {
  if (!stream || !stream.cur) return;
  if (stream.cur.channel === 'thinking') stream.cur.card.classList.remove('open'); // 结束后收起
  stream.cur = null;
}

/* ---------- 实时活动条：当前动作滑动更新（呼吸圆点表明仍在运行） ---------- */

/** 工具参数摘要：JSON 里提取 command/file_path 等有效字段，不显示原始 JSON */
function toolDetail(raw) {
  const s = (raw || '').trim();
  if (s.startsWith('{')) {
    try {
      const o = JSON.parse(s);
      const v = o.command || o.file_path || o.path || o.pattern || o.query || o.url;
      if (typeof v === 'string' && v) return v;
    } catch (_) { /* 截断的 JSON 等，原样返回 */ }
  }
  return s;
}

/** 工具名 → 活动条动词（主路径与 SAGE 节点共用） */
function tickerLabel(name) {
  const nm = (name || '').toLowerCase();
  const en = CUR_LANG === 'en';
  if (nm.includes('bash') || nm.includes('shell') || nm.includes('command') || nm.includes('exec')) {
    return en ? 'Running' : '正在运行';
  }
  if (nm.includes('read')) return en ? 'Reading' : '正在读取';
  if (nm.includes('edit') || nm.includes('write') || nm.includes('patch')) {
    return en ? 'Editing' : '正在编辑';
  }
  if (nm.includes('search') || nm.includes('grep') || nm.includes('glob') || nm.includes('find')) {
    return en ? 'Searching' : '正在搜索';
  }
  return (en ? 'Calling ' : '正在调用 ') + (name || (en ? 'tool' : '工具'));
}

/** st 省略时作用于主流式对象；SAGE 各节点各自传入自己的 ctx 容器，互不抢占。 */
function updateTicker(label, detail, st) {
  const s = st || stream;
  if (!s) return;
  if (!s.ticker) {
    const box = el('div', 'run-ticker');
    box.appendChild(el('span', 'run-ticker-dot'));
    box.appendChild(el('span', 'run-ticker-text'));
    box.title = t('点击展开 / 收起');
    box.addEventListener('click', () => {
      if (!s.ticker) return;
      s.tickerOpen = !s.tickerOpen;
      renderTickerContent(false, s);
    });
    s.ticker = box;
  }
  s.tickerData = { label, detail: detail || '' };
  renderTickerContent(true, s);
  s.ctx.bodyEl.appendChild(s.ticker); // 始终挪到当前末尾
}

/** 按展开状态渲染活动条：收起 = 单行省略；展开 = 完整内容（约 8 行内滚动） */
function renderTickerContent(animate, st) {
  const s = st || stream;
  if (!s || !s.ticker || !s.tickerData) return;
  const d = s.tickerData;
  const box = s.ticker;
  box.classList.toggle('open', !!s.tickerOpen);
  const txt = box.querySelector('.run-ticker-text');
  let pre = box.querySelector('.run-ticker-pre');
  if (s.tickerOpen) {
    txt.textContent = d.label;
    if (!pre) {
      pre = el('pre', 'run-ticker-pre');
      box.appendChild(pre);
    }
    pre.textContent = d.detail || d.label;
  } else {
    if (pre) pre.remove();
    txt.textContent = d.label + (d.detail ? '：' + snippet(d.detail.replace(/\s+/g, ' '), 90) : '');
  }
  if (animate) {
    txt.classList.remove('tick');
    void txt.offsetWidth; // 重触发滑入动画
    txt.classList.add('tick');
  }
}

function removeTicker(st) {
  const s = st || stream;
  if (s && s.ticker) {
    s.ticker.remove();
    s.ticker = null;
  }
}

function finalizeStream() {
  finalizeCur();
  removeTicker();
  // 流式结束的消息也挂操作栏（复制 + 分支——它就是当前最后一条）
  if (stream && stream.finalText && stream.ctx && !stream.ctx.bodyEl.querySelector('.msg-actions')) {
    const bar = appendMsgActions(stream.ctx.bodyEl, stream.finalText.trim());
    if (bar) bar.appendChild(makeForkBtn());
  }
  cursorEl.remove();
  if (stream) {
    stream.ctx.bodyEl.classList.remove('streaming');
    if (stream.usageTimer) clearInterval(stream.usageTimer);
    renderUsageBar(); // 定格最终数值
  }
  stream = null;
}

function ensureStreamText() {
  if (stream.cur && stream.cur.channel === 'text') return stream.cur;
  finalizeCur();
  const target = el('div', 'md');
  stream.ctx.bodyEl.appendChild(target);
  stream.cur = { channel: 'text', raw: '', el: target };
  return stream.cur;
}

function ensureStreamThinking() {
  if (stream.cur && stream.cur.channel === 'thinking') return stream.cur;
  finalizeCur();
  const card = thinkingCard('', true); // 流式中展开，结束后收起
  stream.ctx.bodyEl.appendChild(card);
  stream.cur = { channel: 'thinking', raw: '', card, body: card.querySelector('.think-body') };
  return stream.cur;
}

function placeCursorIn(container) {
  let target = container.lastElementChild;
  if (target && target.tagName === 'PRE') target = target.firstElementChild || target;
  (target || container).appendChild(cursorEl);
}

function appendStderr(line) {
  if (!stream.stderrPre) {
    const card = el('div', 'card stderr');
    const head = el('div', 'card-head');
    head.appendChild(el('span', 'card-caret', '▸'));
    head.appendChild(el('span', 'card-title', '⚠ stderr'));
    const body = el('div', 'card-body');
    const pre = el('pre', 'io-pre');
    body.appendChild(pre);
    card.appendChild(head);
    card.appendChild(body);
    head.addEventListener('click', () => card.classList.toggle('open'));
    stream.ctx.bodyEl.appendChild(card);
    stream.stderrPre = pre;
  }
  stream.stderrPre.textContent += (stream.stderrPre.textContent ? '\n' : '') + line;
}

function stderrCard(title, lines) {
  const card = el('div', 'card stderr');
  const head = el('div', 'card-head');
  head.appendChild(el('span', 'card-caret', '▸'));
  head.appendChild(el('span', 'card-title', title));
  const body = el('div', 'card-body');
  const pre = el('pre', 'io-pre');
  pre.textContent = lines.join('\n');
  body.appendChild(pre);
  card.appendChild(head);
  card.appendChild(body);
  head.addEventListener('click', () => card.classList.toggle('open'));
  return card;
}

/** 统一事件处理（CONTRACT §3.3 的 9 种事件） */
function handleEvent(ev) {
  if (!stream || !ev || typeof ev !== 'object') return;
  const near = chatNearBottom();
  switch (ev.t) {
    case 'run':
      state.runId = ev.run_id;
      if (ev.run_id) state.runIds.add(ev.run_id);
      break;
    case 'init':
      // 新会话拿到 id，或 /fork 分叉出了新 id
      if (ev.session_id && state.session && state.session.id !== ev.session_id) {
        state.session.id = ev.session_id;
        prependConvRow();
        renderProjects(); // 项目分组里也立即出现（合并运行中占位）
      }
      // 路由决策在拿到会话 id 的瞬间立即持久化——中途刷新页面也不丢
      if (state.pendingSage && state.session && state.session.id) {
        sageStoreSave(state.session.agent + ':' + state.session.id, state.pendingSage);
        state.pendingSage = null;
      }
      break;
    case 'delta':
      if (!ev.text) break; // 空增量不创建也不追加块（claude 回合开头会先发 text 为空串的 delta）
      removeTicker(); // 正文/思考流本身就是可见进度
      endToolGroup(stream.ctx);
      if (ev.channel === 'thinking') {
        const b = ensureStreamThinking();
        b.raw += ev.text || '';
        b.body.textContent = b.raw;
        b.body.appendChild(cursorEl);
      } else {
        const b = ensureStreamText();
        b.raw += ev.text || '';
        renderMarkdown(b.el, b.raw);
        placeCursorIn(b.el);
        stream.finalText = (stream.finalText + ev.text).slice(-12000);
      }
      break;
    case 'text': {
      finalizeCur();
      removeTicker();
      endToolGroup(stream.ctx);
      const d = el('div', 'md fade-in');
      renderMarkdown(d, ev.text || '');
      stream.ctx.bodyEl.appendChild(d);
      stream.ctx.bodyEl.appendChild(cursorEl);
      stream.finalText = (stream.finalText + '\n' + (ev.text || '')).slice(-12000);
      break;
    }
    case 'thinking':
      finalizeCur();
      removeTicker();
      endToolGroup(stream.ctx);
      stream.ctx.bodyEl.appendChild(thinkingCard(ev.text || '', false));
      stream.ctx.bodyEl.appendChild(cursorEl);
      break;
    case 'plan':
      finalizeCur();
      upsertPlan(stream.ctx, ev.items || []);
      stream.ctx.bodyEl.appendChild(cursorEl);
      break;
    case 'user_echo': {
      // 重连补显本轮用户消息：转录已含（CLI 已落盘）则跳过，避免重复。
      // 比对用剥掉图片标记后的正文——气泡里显示的就是这一份。
      const shown = visibleUserText(ev.text);
      const bubbles = chatMsgs.querySelectorAll('.msg-user .bubble');
      const last = bubbles[bubbles.length - 1];
      if (shown && !(last && last.textContent.trim() === shown)) {
        renderUserMsg(chatMsgs, [{ kind: 'text', text: shown }], null);
        const asst = stream.ctx && stream.ctx.bodyEl;
        const bubble = chatMsgs.lastElementChild;
        if (asst && bubble && bubble !== asst) chatMsgs.insertBefore(bubble, asst);
        scrollChat();
      }
      break;
    }
    case 'sub_text':
      appendSubEvent(ev.sub, 'text', '', ev.text || '');
      break;
    case 'sub_tool':
      appendSubEvent(ev.sub, 'tool', ev.name || '工具', ev.text || '');
      break;
    case 'tool_use': {
      finalizeCur();
      appendToolUse(stream.ctx, ev.name || '工具', ev.text || '', ev.id);
      // 实时活动条：滑动展示当前正在执行的动作
      updateTicker(tickerLabel(ev.name || ''), toolDetail(ev.text || ''));
      stream.ctx.bodyEl.appendChild(cursorEl);
      break;
    }
    case 'tool_result':
      finalizeCur();
      appendToolResult(stream.ctx, ev.text || '');
      stream.ctx.bodyEl.appendChild(cursorEl);
      break;
    case 'file_edit':
      if (ev.path) {
        stream.ctx.fileEdits.add(ev.path);
        updateTicker(CUR_LANG === 'en' ? 'Editing' : '正在编辑', ev.path);
      }
      break;
    case 'usage': {
      const u = stream.usage;
      if (ev.mode === 'set') {
        u.input = ev.input || 0;
        u.output = ev.output || 0;
        u.cr = ev.cache_read || 0;
        u.cw = ev.cache_write || 0;
        // scope=session（codex 回放文件旁路）：整场权威值，含续聊前的全部
        if (ev.scope === 'session') {
          u.abs = true;
          // 本轮速度基点：首个权威值的输出量与时刻
          if (!stream.spd0) stream.spd0 = { out: u.output, t: Date.now() };
        }
      } else {
        u.input += ev.input || 0;
        u.output += ev.output || 0;
        u.cr += ev.cache_read || 0;
        u.cw += ev.cache_write || 0;
      }
      if (ev.context) u.ctx = ev.context;
      if (ev.window) u.window = ev.window;
      u.has = true;
      try {
        renderUsageBar(); // 展示层异常绝不打断流式读取
      } catch (_) { /* 忽略 */ }
      break;
    }
    case 'status':
      finalizeCur();
      stream.ctx.bodyEl.appendChild(el('div', 'status-line', '⏳ ' + (ev.text || '')));
      stream.ctx.bodyEl.appendChild(cursorEl);
      break;
    case 'stderr':
      appendStderr(ev.text || '');
      break;
    case 'done':
      if (ev.session_id && state.session && state.session.id !== ev.session_id) {
        state.session.id = ev.session_id;
        prependConvRow();
      }
      stream.doneOk = !!ev.ok;
      stream.doneError = ev.error || ''; // 供 SAGE 判定是模型级不可用还是普通失败
      finalizeCur();
      flushFilesCard(stream.ctx); // 汇总本轮编辑的文件卡片
      if (!ev.ok) {
        if (ev.error === '已停止') {
          stream.ctx.bodyEl.appendChild(el('div', 'status-line', t('■ 已停止')));
        } else {
          stream.ctx.bodyEl.appendChild(el('div', 'error-bar', ev.error || t('运行失败（无错误信息）')));
        }
      } else if (stream.startedAt && Date.now() - stream.startedAt > 3000) {
        stream.ctx.bodyEl.appendChild(
          el('div', 'done-line', t('已处理') + ' ' + fmtDuration(Date.now() - stream.startedAt))
        );
      }
      break;
    default:
      break; // 未知事件忽略
  }
  if (near) scrollChat();
}

/** POST /api/chat 并逐行消费 NDJSON（行缓冲正确处理 chunk 跨行与残尾） */
async function streamChat(req, onEvent, signal) {
  const resp = await fetch('/api/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
    signal,
  });
  if (!resp.ok) {
    let msg = 'HTTP ' + resp.status;
    try {
      const j = await resp.json();
      if (j && j.error) msg = j.error;
    } catch (_) { /* 保留状态码 */ }
    throw new Error(msg);
  }
  if (!resp.body) throw new Error('当前浏览器不支持流式响应');
  await readNdjson(resp, onEvent);
}

/** 逐行消费一个 NDJSON 响应体 */
async function readNdjson(resp, onEvent) {
  const reader = resp.body.getReader();
  const decoder = new TextDecoder('utf-8');
  let buffer = '';
  const feed = (line) => {
    line = line.trim(); // 兼容 \r\n
    if (!line) return;
    let ev;
    try {
      ev = JSON.parse(line);
    } catch (_) {
      return; // 非 JSON 行忽略
    }
    onEvent(ev);
  };
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop(); // 最后一段可能不完整，留在缓冲
    for (const l of lines) feed(l);
  }
  buffer += decoder.decode(); // 冲刷解码器残留
  if (buffer) feed(buffer);   // 流结束时残余缓冲也要处理
}

/* ---------- 发送 ---------- */

function showComposerError(msg) {
  const box = $('#composer-error');
  box.textContent = msg;
  box.classList.remove('hidden');
}

function hideComposerError() {
  $('#composer-error').classList.add('hidden');
}

function setSendButton(streaming) {
  const btn = $('#send-btn');
  btn.classList.toggle('stop', streaming);
  btn.disabled = state.stopping;
  btn.title = state.stopping ? '正在停止…' : streaming ? '停止' : '发送';
  $('#send-icon').classList.toggle('hidden', streaming);
  $('#stop-icon').classList.toggle('hidden', !streaming);
}

/* ---------- 顶部 toast 通知 ---------- */

let toastEl = null;

function showToast(msg) {
  if (toastEl) toastEl.remove();
  toastEl = el('div', 'toast', msg);
  document.body.appendChild(toastEl);
  const cur = toastEl;
  setTimeout(() => {
    cur.classList.add('gone');
    setTimeout(() => cur.remove(), 350);
  }, 3200);
}

/* ---------- 协作会话关联持久化（刷新后把子会话缝合回主会话视图） ---------- */

function collabStoreLoad() {
  try {
    return JSON.parse(localStorage.getItem('ah-collab')) || { links: {}, back: {} };
  } catch (_) {
    return { links: {}, back: {} };
  }
}

function collabLinkSave(primaryKey, entry, partnerKey) {
  const st = collabStoreLoad();
  st.links[primaryKey] = st.links[primaryKey] || [];
  // 幂等：同一子会话只记一条（init 早存 + 结束兜底存会各调一次）
  const existing = st.links[primaryKey].find((e) => e.partner === entry.partner);
  if (existing) {
    Object.assign(existing, entry);
  } else {
    st.links[primaryKey].push(entry);
  }
  // 返回入口归属首个主会话（分叉复制关联时不抢走父会话的归属）
  if (!st.back[partnerKey]) st.back[partnerKey] = primaryKey;
  const keys = Object.keys(st.links);
  while (keys.length > 80) {
    delete st.links[keys.shift()]; // 只留最近 80 个主会话的关联
  }
  localStorage.setItem('ah-collab', JSON.stringify(st));
}

function collabTaskKey(text) {
  return String(text || '')
    .split('请查看图片文件')[0]
    .replace(/\s+/g, ' ')
    .trim();
}

function executorRuntime(label) {
  const value = String(label || '').trim();
  if (value.startsWith('Claude Code')) return 'claude';
  if (value.startsWith('Codex')) return 'codex';
  return '';
}

/** 从历史 transcript.sage 的不可见元数据恢复完整协作关系。
 *  workflow id 优先精确配对；旧记录没有 id 时退化为原始任务 + owner/executor 语义匹配。 */
async function discoverCollabLinks(s, tr, key) {
  // 分叉会话不做回溯配对：继承内容与父会话相同，会把父会话的子会话错配过来
  const isFork = (tr.messages || []).some((m) =>
    (m.blocks || []).some((b) => b.kind === 'divider' && (b.text || '').includes('分支点'))
  );
  if (isFork) return [];
  const mainMetas = (tr.sage || []).filter(
    (meta) => meta && ['collaborate', 'summary'].includes(meta.kind) && meta.original_task
  );
  const latestMeta = mainMetas[mainMetas.length - 1] || null;
  const fallbackUser = [...(tr.messages || [])]
    .reverse()
    .find((message) =>
      message.role === 'user' &&
      (message.blocks || []).some((block) => block.kind === 'text' && block.text)
    );
  const fallbackTask = fallbackUser
    ? ((fallbackUser.blocks || []).find((block) => block.kind === 'text' && block.text) || {}).text
    : '';
  const task = latestMeta && latestMeta.original_task || fallbackTask || '';
  const taskKey = collabTaskKey(task);
  const probe = taskKey.slice(0, 60);
  if (probe.length < 8) return [];
  const workflowIds = new Set(
    mainMetas
      .filter((meta) => collabTaskKey(meta.original_task) === taskKey && meta.workflow_id)
      .map((meta) => meta.workflow_id)
  );
  let sessions;
  try {
    sessions = await api.get(
      '/api/sessions?' + new URLSearchParams({ project: s.project || '', limit: '200' })
    );
  } catch (_) {
    return [];
  }
  const titleKey = collabTaskKey(task.split(/\r?\n/).find((line) => line.trim()) || task).slice(0, 36);
  // 先按原始任务标题缩小候选；实际归属由 transcript.sage 元数据决定。
  const back0 = collabStoreLoad().back;
  const cands = sessions
    .filter((x) => {
      if (x.id === s.id) return false;
      const titleMatch = SageScheduler.shouldInspectCollabCandidate(
        collabTaskKey(x.title), titleKey, workflowIds.size > 0
      );
      const legacyHandoff = SageScheduler.isLegacyHandoffSource(s, x, sessions);
      if (!titleMatch && !legacyHandoff) return false;
      const owner = back0[x.agent + ':' + x.id];
      return !owner || owner === key;
    })
    .slice(0, workflowIds.size ? 50 : 20);
  const found = [];
  for (const c of cands) {
    try {
      const qs = new URLSearchParams({ agent: c.agent, id: c.id, project: s.project || '' });
      const sub = await api.get('/api/session?' + qs);
      const handoffMetas = (sub.sage || []).filter(
        (meta) =>
          meta && meta.kind === 'handoff' &&
          meta.source_agent === s.agent && meta.source_session_id === s.id
      );
      const legacyHandoffMetas = handoffMetas.length
        ? []
        : (sub.sage || []).filter(
            (meta) =>
              meta && meta.kind === 'handoff' &&
              !meta.source_session_id && SageScheduler.isLegacyHandoffSource(s, c, sessions)
          );
      const matchedHandoff = handoffMetas.length ? handoffMetas : legacyHandoffMetas;
      if (matchedHandoff.length) {
        const meta = matchedHandoff[matchedHandoff.length - 1];
        found.push({
          partner: c.agent + ':' + c.id,
          label: meta.executor || (AGENTS[c.agent] ? AGENTS[c.agent].label : c.agent),
          kind: 'handoff',
          cats: '',
          workflow_id: meta.workflow_id || null,
          model: sub.usage && sub.usage.model || null,
          ts: Date.parse(c.created || '') || 0,
        });
        continue;
      }
      let metas = (sub.sage || []).filter(
        (meta) =>
          meta && meta.kind === 'collaborate' &&
          collabTaskKey(meta.original_task).includes(probe) &&
          executorRuntime(meta.owner) === s.agent &&
          meta.executor && meta.owner && meta.executor !== meta.owner
      );
      if (workflowIds.size) {
        const exact = metas.filter((meta) => workflowIds.has(meta.workflow_id));
        if (exact.length) metas = exact;
        else if (metas.some((meta) => meta.workflow_id)) continue;
      }
      if (!metas.length) continue;
      const cats = [...new Set(metas.map((meta) => meta.requirement).filter(Boolean))];
      const meta = metas[metas.length - 1];
      found.push({
        partner: c.agent + ':' + c.id,
        label: meta.executor || (AGENTS[c.agent] ? AGENTS[c.agent].label : c.agent),
        kind: 'pipeline',
        cats: cats.join('、'),
        workflow_id: meta.workflow_id || null,
        model: sub.usage && sub.usage.model || null,
        ts: Date.parse(c.created || '') || 0,
      });
    } catch (_) {
      /* 单个候选失败跳过 */
    }
  }
  found.sort((a, b) => a.ts - b.ts || a.partner.localeCompare(b.partner));
  for (const ln of found) {
    collabLinkSave(key, ln, ln.partner); // 回写，下次免扫描
  }
  return found;
}

/** 子会话最新结论回注主会话：跳回主会话，让主 agent 消化并落实后续 */
async function feedBackToPrimary() {
  if (state.streaming || !state.backPrimary || !state.session) return;
  // 子会话还在运行时不回注（会拿到半截结论）
  const running = state.runsIndex[state.session.id];
  if (running && running.running) {
    showToast(CUR_LANG === 'en' ? 'Sub-session still running — wait for it to finish' : '子会话还在运行中，等它跑完再回注');
    return;
  }
  const sub = {
    agent: state.session.agent,
    id: state.session.id,
    project: state.session.project,
  };
  const subLabel = AGENTS[sub.agent] ? AGENTS[sub.agent].label : sub.agent;
  const pk = state.backPrimary;
  let finalTxt = '';
  try {
    const qs = new URLSearchParams({ agent: sub.agent, id: sub.id, project: sub.project || '' });
    finalTxt = transcriptFinalText(await api.get('/api/session?' + qs));
  } catch (_) {
    /* 取不到就落到空 */
  }
  if (!finalTxt) {
    showToast(CUR_LANG === 'en' ? 'Nothing to feed back yet' : '子会话暂无可回注的结论');
    return;
  }
  const i = pk.indexOf(':');
  const prim = { agent: pk.slice(0, i), id: pk.slice(i + 1), project: sub.project, title: '' };
  await openSession(prim);
  if (!state.session || state.session.id !== prim.id || state.session.agent !== prim.agent) return;
  await runPrimaryFollowup(
    { agent: prim.agent, id: prim.id, project: prim.project },
    (CUR_LANG === 'en' ? '🤝 Consolidate · ' : '🤝 汇总回注 · ') +
      (AGENTS[prim.agent] ? AGENTS[prim.agent].label : prim.agent) +
      (CUR_LANG === 'en' ? ' wraps up' : ' 收尾'),
    '【协作汇总】搭档 agent（' + subLabel + '）在子会话中的最新结论如下：\n\n' + finalTxt +
      '\n\n请消化这份结论并在本会话中落实后续（需要实现或修改的直接进行），最后给出状态确认。'
  );
}

/* ---------- 右侧协作子会话面板：状态 + 入口（子会话不进侧栏） ---------- */

function renderCollabPanel() {
  const panel = $('#collab-panel');
  if (!panel) return;
  const s = state.session;
  const links = s && s.id ? collabStoreLoad().links[s.agent + ':' + s.id] || [] : [];
  const linkedIds = new Set(
    links.map((link) => link.partner.slice(link.partner.indexOf(':') + 1))
  );
  const globalRuns = Object.values(state.runsIndex).filter((run) => run && run.running);
  const otherRuns = globalRuns.filter(
    (run) => run.session_id && run.session_id !== (s && s.id) && !linkedIds.has(run.session_id)
  );
  if (!links.length && !otherRuns.length) {
    panel.classList.add('hidden');
    return;
  }
  panel.textContent = '';
  const globalRunning = globalRuns.length;
  panel.appendChild(
    el(
      'div',
      'collab-panel-title',
      CUR_LANG === 'en'
        ? `🤝 ${links.length} sub-session${links.length === 1 ? '' : 's'} · ${globalRunning} running globally`
        : `🤝 当前任务 ${links.length} 个子会话 · 全局运行 ${globalRunning}`
    )
  );
  for (const ln of links) {
    const i = ln.partner.indexOf(':');
    const pAgent = ln.partner.slice(0, i);
    const pId = ln.partner.slice(i + 1);
    const row = el('div', 'collab-panel-row');
    const dot = el('span', 'agent-dot ' + (AGENTS[pAgent] ? AGENTS[pAgent].cls : ''));
    dot.appendChild(agentIcon(pAgent, 13));
    row.appendChild(dot);
    const main = el('span', 'collab-panel-main');
    main.appendChild(el('span', 'collab-panel-name', ln.label));
    const nodes = ln.cats
      ? (CUR_LANG === 'en' ? `Nodes: ${ln.cats}` : `节点：${ln.cats}`)
      : ln.kind === 'handoff'
        ? (CUR_LANG === 'en' ? 'Full task ownership' : '完整任务接管')
        : ln.kind === 'review' ? t('复查') : t('协作执行');
    main.appendChild(el('span', 'collab-panel-nodes', nodes));
    row.appendChild(main);
    // 状态只认运行记录本身：没有记录就是「未知」，不能默认报「已完成」。
    const r = state.runsIndex[pId];
    let stTxt;
    let stCls;
    if (!r) {
      stTxt = CUR_LANG === 'en' ? '—' : '—';
      stCls = 'unknown';
    } else if (r.running) {
      stTxt = CUR_LANG === 'en' ? 'running' : '运行中';
      stCls = 'run';
    } else if (r.error === '已停止') {
      stTxt = CUR_LANG === 'en' ? 'stopped' : '已停止';
      stCls = 'stopped';
    } else if (r.ok === false) {
      stTxt = CUR_LANG === 'en' ? 'error' : '出错';
      stCls = 'err';
    } else {
      stTxt = CUR_LANG === 'en' ? 'done' : '已完成';
      stCls = 'ok';
    }
    row.appendChild(el('span', 'collab-panel-st ' + stCls, stTxt));
    row.title = stCls === 'err'
      ? (r.error || (CUR_LANG === 'en' ? 'Sub-session failed' : '子会话执行失败'))
      : stCls === 'unknown'
        ? (CUR_LANG === 'en'
            ? 'No run record for this sub-session — click to open'
            : '本次未取到该子会话的运行记录，点击打开查看')
        : CUR_LANG === 'en' ? 'Open sub-session' : '点击打开子会话';
    row.addEventListener('click', () =>
      openSession({
        agent: pAgent, id: pId, project: s.project, title: '',
        model: ln.model || null, effort: ln.effort || null,
      })
    );
    panel.appendChild(row);
  }
  if (otherRuns.length) {
    panel.appendChild(
      el(
        'div',
        'collab-panel-section',
        CUR_LANG === 'en' ? `Other tasks running · ${otherRuns.length}` : `其他任务运行中 · ${otherRuns.length}`
      )
    );
    for (const run of otherRuns) {
      const row = el('div', 'collab-panel-row collab-panel-global');
      const agent = run.agent || 'codex';
      const dot = el('span', 'agent-dot ' + (AGENTS[agent] ? AGENTS[agent].cls : ''));
      dot.appendChild(agentIcon(agent, 13));
      row.appendChild(dot);
      const main = el('span', 'collab-panel-main');
      main.appendChild(
        el('span', 'collab-panel-name', run.sage && run.sage.executor || AGENTS[agent]?.label || agent)
      );
      main.appendChild(
        el(
          'span',
          'collab-panel-nodes',
          run.sage && run.sage.requirement
            ? (CUR_LANG === 'en' ? `Node: ${run.sage.requirement}` : `节点：${run.sage.requirement}`)
            : snippet(run.prompt || (CUR_LANG === 'en' ? 'Running task' : '运行中任务'), 34)
        )
      );
      row.appendChild(main);
      row.appendChild(el('span', 'collab-panel-st run', CUR_LANG === 'en' ? 'running' : '运行中'));
      row.title = CUR_LANG === 'en' ? 'Open running session' : '点击打开运行中的会话';
      row.addEventListener('click', () =>
        openSession({
          agent, id: run.session_id, project: run.project || (s && s.project) || '',
          title: run.prompt || '', model: null, effort: null,
        })
      );
      panel.appendChild(row);
    }
  }
  panel.classList.remove('hidden');
}

/** 转录的最终回答文本（末条有实质内容的助手消息，补回注用） */
function transcriptFinalText(tr) {
  const asst = (tr.messages || []).filter((m) => m.role === 'assistant');
  for (let i = asst.length - 1; i >= 0; i--) {
    const txt = (asst[i].blocks || [])
      .filter((b) => b.kind === 'text' && b.text)
      .map((b) => b.text)
      .join('\n')
      .trim();
    if (txt.length > 20) return txt.slice(0, 12000);
  }
  return '';
}

/** 移交结论卡：把接管方的最终回答摊在来源会话里（默认收起，点开看全文）。
 *  只做展示，不喂给主 agent——移交的语义就是它已经不管这件事了。 */
function handoffResultCard(tr, partnerId, link) {
  const run = state.runsIndex[partnerId];
  const en = CUR_LANG === 'en';
  const card = el('div', 'card handoff-result');
  const head = el('div', 'card-head');
  head.appendChild(el('span', 'card-caret', '▸'));
  if (run && run.running) {
    // 还在跑就别贴半截结论，给个状态占位，刷新/重开时自然会变成结论
    head.appendChild(
      el('span', 'card-title', (en ? '🧭 Taken over · running · ' : '🧭 已接管 · 运行中 · ') + link.label)
    );
    card.appendChild(head);
    card.classList.add('empty-note');
    return card;
  }
  const text = transcriptFinalText(tr);
  if (!text) return null;
  head.appendChild(
    el('span', 'card-title', (en ? '🧭 Handoff result · ' : '🧭 接管结论 · ') + link.label)
  );
  const body = el('div', 'card-body');
  const md = el('div', 'md');
  renderMarkdown(md, text);
  body.appendChild(md);
  card.appendChild(head);
  card.appendChild(body);
  head.addEventListener('click', () => card.classList.toggle('open'));
  return card;
}

/** 本页观察到过「运行中」的子会话 id：其转为完成时触发自动回注 */
const seenRunningPartners = new Set();

/** 标记某条协作关联已完成回注（防多窗口/重复触发） */
function collabLinkMarkFed(primaryKey, partnerKey) {
  const st = collabStoreLoad();
  for (const e of st.links[primaryKey] || []) {
    if (e.partner === partnerKey) e.fed = true;
  }
  localStorage.setItem('ah-collab', JSON.stringify(st));
}

/** 打开会话时缝合协作关联：主会话内联子会话内容；子会话给出主会话入口。
 *  幂等：可在运行状态变化时重复调用刷新（先清旧节点再重建）。 */
async function stitchCollab(s, tr) {
  const key = s.agent + ':' + s.id;
  const st = collabStoreLoad();
  chatMsgs.querySelectorAll('.collab-stitch, .divider.collab-jump').forEach((n) => n.remove());
  const primaryKey = st.back[key];
  if (primaryKey) {
    const backEntry = (st.links[primaryKey] || []).find((entry) => entry.partner === key);
    const handoffBack = backEntry && backEntry.kind === 'handoff';
    const d = renderDivider(
      handoffBack
        ? (CUR_LANG === 'en'
            ? '🧭 SAGE handoff session · click to open the source session'
            : '🧭 SAGE 移交会话 · 点击打开来源会话')
        : (CUR_LANG === 'en'
            ? '🤝 Collab sub-session · click to open the main session'
            : '🤝 协作子会话 · 点击打开主会话')
    );
    d.classList.add('collab-jump');
    d.addEventListener('click', () => {
      const i = primaryKey.indexOf(':');
      openSession({
        agent: primaryKey.slice(0, i),
        id: primaryKey.slice(i + 1),
        project: s.project,
        title: '',
      });
    });
    chatMsgs.insertBefore(d, chatMsgs.firstChild);
  }
  let links = st.links[key] ? [...st.links[key]] : [];
  if (tr) {
    await discoverCollabLinks(s, tr, key); // 每次补扫 metadata，修复缺失/新增 partner
    if (!state.session || state.session.id !== s.id || state.session.agent !== s.agent) return;
    links = [...(collabStoreLoad().links[key] || [])];
  }
  renderCollabPanel(); // 回溯配对可能新增关联 → 刷新右侧面板
  if (!links.length) return;
  await ensureRunsIndex(); // 补回注按钮需要知道子会话是否还在运行
  for (const ln of links) {
    try {
      const i = ln.partner.indexOf(':');
      const pId = ln.partner.slice(i + 1);
      const qs = new URLSearchParams({
        agent: ln.partner.slice(0, i),
        id: pId,
        project: s.project || '',
      });
      const tr = await api.get('/api/session?' + qs);
      if (!state.session || state.session.id !== s.id || state.session.agent !== s.agent) return;
      const sec = el('div', 'collab-stitch');
      const title =
        ln.kind === 'pipeline'
          ? (CUR_LANG === 'en' ? '🤝 Division of work · ' : '🤝 分工执行 · ') +
            ln.label + (ln.cats ? '（' + ln.cats + '）' : '')
          : ln.kind === 'handoff'
            ? (CUR_LANG === 'en' ? '🧭 SAGE handoff · ' : '🧭 SAGE 移交 · ') + ln.label
          : (CUR_LANG === 'en' ? '🤝 Collaborative review · ' : '🤝 协作复查 · ') + ln.label;
      // 默认收起：只展示入口分隔线（子会话内容不内联铺开），点击直达子会话
      const secDiv = renderDivider(
        title + (CUR_LANG === 'en' ? ' · click to enter sub-session' : ' · 点击进入子会话')
      );
      secDiv.classList.add('collab-jump');
      const pk = ln.partner;
      secDiv.addEventListener('click', () => {
        const j = pk.indexOf(':');
        openSession({
          agent: pk.slice(0, j), id: pk.slice(j + 1), project: s.project,
          title: '', model: ln.model || null, effort: ln.effort || null,
        });
      });
      sec.appendChild(secDiv);
      // 移交仍然不回注（所有权已经交出去了，再让主 agent 跑一轮是白烧一轮 token），
      // 但结论要能在这里直接看到——否则主会话只剩一条空分隔线，任务像是凭空消失了。
      if (ln.kind === 'handoff') {
        const card = handoffResultCard(tr, pId, ln);
        if (card) sec.appendChild(card);
        chatMsgs.appendChild(sec);
        scrollChat();
        continue;
      }
      // 插到对应的回注卡之前（时间序一一对应；找不到则追加到末尾）
      const anchors = [...chatMsgs.querySelectorAll('.collab-inject')].filter((c) =>
        /分工产出回注|复查意见回注|Consolidated partner output|Review feedback/.test(c.textContent || '')
      );
      const anchor = anchors[links.indexOf(ln)] || null;
      if (anchor && anchor.parentNode === chatMsgs) {
        chatMsgs.insertBefore(sec, anchor);
      } else {
        chatMsgs.appendChild(sec);
        // 官方 SAGE DAG（带 workflow_id）的收尾由所有者的 __summary__ 节点完成，
        // 再走一遍旧版回注等于让主 agent 白跑一轮，只保留子会话入口。
        if (ln.workflow_id) {
          scrollChat();
          continue;
        }
        // 无回注锚点且子会话已结束 → 回注收尾。本页刚观察到它「运行中→完成」
        // 且未回注过 = 活跃流水线的延续 → 自动回注；否则（陈年旧会话）留手动按钮。
        const pr = state.runsIndex[pId];
        const running = pr && pr.running;
        const finalTxt = running ? '' : transcriptFinalText(tr);
        if (finalTxt) {
          const primaryLabel = AGENTS[s.agent] ? AGENTS[s.agent].label : s.agent;
          const fbPrompt =
            ln.kind === 'pipeline'
              ? '【协作汇总】搭档 agent（' + ln.label + '）已完成其分工' +
                (ln.cats ? '（' + ln.cats + '）' : '') + '，产出如下：\n\n' + finalTxt +
                '\n\n请核对搭档产出与你的实现是否一致：有出入的直接修正，并给出本次任务的最终总结。'
              : '【协作复查回注】搭档 agent（' + ln.label + '）对你上一轮工作的只读复查意见如下：\n\n' +
                finalTxt + '\n\n请核对以上意见：确认无误的部分简要说明；确有问题的部分直接修正并说明改动。';
          const fbDivider =
            (CUR_LANG === 'en' ? '🤝 Consolidate · ' : '🤝 汇总回注 · ') + primaryLabel +
            (CUR_LANG === 'en' ? ' wraps up' : ' 收尾');
          const fire = async () => {
            collabLinkMarkFed(key, ln.partner); // 先标记，防多窗口重复回注
            await runPrimaryFollowup({ agent: s.agent, id: s.id, project: s.project }, fbDivider, fbPrompt);
          };
          if (!ln.fed && seenRunningPartners.has(pId) && !state.streaming) {
            seenRunningPartners.delete(pId);
            showToast(
              CUR_LANG === 'en'
                ? '🤝 Sub-session finished — consolidating back automatically'
                : '🤝 子会话完成，自动回注收尾中…'
            );
            fire();
          } else {
            const btn = el(
              'button',
              'btn-ghost collab-feed',
              CUR_LANG === 'en'
                ? '▶ Feed this back to the main agent to wrap up'
                : '▶ 补回注：把该结论交给主 agent 收尾'
            );
            btn.type = 'button';
            btn.addEventListener('click', () => {
              if (state.streaming) return;
              btn.remove();
              fire();
            });
            sec.appendChild(btn);
          }
        }
      }
      scrollChat();
    } catch (_) {
      /* 子会话取不到（被删等）就跳过 */
    }
  }
}

/* ---------- 路由决策按会话持久化（刷新/重开会话后重现决策卡） ---------- */

function sageStoreLoad() {
  try {
    return JSON.parse(localStorage.getItem('ah-sage-decisions')) || {};
  } catch (_) {
    return {};
  }
}

function sageStoreSave(key, d) {
  const all = sageStoreLoad();
  // 展示不需要 decision_blob（体积大），去掉再存
  const lean = { ...d };
  delete lean.decision_blob;
  delete all[key];
  all[key] = lean;
  const keys = Object.keys(all);
  while (keys.length > 120) delete all[keys.shift()]; // 只留最近 120 条
  localStorage.setItem('ah-sage-decisions', JSON.stringify(all));
}

/** 运行期确认「当前 provider 不提供这个模型」的记忆：executorId → 记录时间戳。
 *  服务端已按网关 /models 过滤过一遍，这里兜住真正跑起来才暴露的型号（网关探测降级、
 *  或网关列了但实际不可用）。到期自动放行，模型重新上线不需要手工清。 */
const MODEL_BLOCK_KEY = 'ah-model-blocked';
const MODEL_BLOCK_TTL = 24 * 3600 * 1000;

function modelBlockLoad() {
  let all = {};
  try {
    all = JSON.parse(localStorage.getItem(MODEL_BLOCK_KEY)) || {};
  } catch (_) {
    all = {};
  }
  const now = Date.now();
  let expired = false;
  for (const [id, ts] of Object.entries(all)) {
    if (!(now - ts < MODEL_BLOCK_TTL)) {
      delete all[id];
      expired = true;
    }
  }
  if (expired) localStorage.setItem(MODEL_BLOCK_KEY, JSON.stringify(all));
  return all;
}

function modelBlockList() {
  return Object.keys(modelBlockLoad());
}

/** 新入黑名单返回 true（已在名单内返回 false，避免重复提示）。 */
function modelBlockAdd(executorId) {
  const all = modelBlockLoad();
  if (all[executorId]) return false;
  all[executorId] = Date.now();
  localStorage.setItem(MODEL_BLOCK_KEY, JSON.stringify(all));
  return true;
}

/** provider 层面的「这个模型用不了」：换思考等级或重试都救不回来，只能换执行者。 */
function isModelUnavailableError(text) {
  const s = String(text || '').toLowerCase();
  return (
    s.includes('model_not_found') ||
    s.includes('unknown provider for model') ||
    s.includes('unsupported model') ||
    s.includes('does not have access to model') ||
    (s.includes('model') && s.includes('does not exist'))
  );
}

/** 失败后的路由记忆：executor 进本任务 failed_agents；模型级不可用再进全局黑名单。 */
function sageRememberFailure(executorId, errorText) {
  const task = state.sageTaskText;
  if (!executorId || !task) return;
  const f =
    state.sageFailed && state.sageFailed.task === task ? state.sageFailed : { task, agents: [] };
  if (!f.agents.includes(executorId)) f.agents.push(executorId);
  state.sageFailed = f;
  if (isModelUnavailableError(errorText) && modelBlockAdd(executorId)) {
    showToast(
      CUR_LANG === 'en'
        ? `⚠ The provider does not serve ${executorId} — excluded from routing`
        : `⚠ 当前 provider 不提供 ${executorId}，已从路由候选中排除`
    );
  }
}

/** 会话已经积累了多少「换人就得重来」的工作。
 *  上游 ALGORITHM.md：switching loss 取决于 progress 与 transferable context，
 *  「harder after valuable non-transferable work has accumulated」。
 *  这里此前硬编码 progress=0，等于永远宣称毫无积累，把上游的防抖整个关掉了。
 *  用上下文占用比当代理指标：聊得越久、上下文越满，换人越亏。 */
function sageSessionProgress(session) {
  if (!session || !session.id) return 0; // 新会话，确实没有积累
  const u = state.histUsage;
  const ratio = u && u.context > 0 && u.window > 0 ? u.context / u.window : 0;
  // 已经在这条会话里干过活就不该按 0 算，给一个下限
  return Math.max(0.25, Math.min(0.9, ratio));
}

/** 官方 Task/ExecutionState 输入：未配置真实 SLA 时不伪造 budget/deadline。 */
function sageRouteContext(session, failed) {
  const previous =
    session && session.id ? sageStoreLoad()[session.agent + ':' + session.id] || null : null;
  const runtime = session ? session.agent : state.agent;
  const modelInfo = state.modelsInfo && state.modelsInfo[runtime];
  const incumbentModel = session && session.model || state.model || modelInfo && modelInfo.default || null;
  return {
    failed: failed || [],
    state: previous
      ? {
          active_agents: previous.agents || [session.agent],
          active_mode: previous.mode || 'self',
          completed_requirements: [], // 追问是新 Task，不沿用上一 Task 的完成节点
          progress: sageSessionProgress(session),
          // null = 用上游默认 0.70。此前写死 1（「上下文完全可无损转移」）纯属误读：
          // 跨 CLI 移交要把来源记录重灌一遍，本来就是有损的。
          transferable_context: null,
          failed_agents: failed || [],
          failure_count: (failed || []).length,
        }
      : {
          // 没有路由记录 ≠ 没有在任执行者：这条会话正跑着的就是 session.agent。
          // 传空数组会命中上游 _switch_loss 的「无 active route」分支，移交成本直接归零，
          // 于是任何一条没被 SAGE 决策过的会话（终端建的、刚打开智能路由的）追问必漂移。
          active_agents: session && session.id ? [session.agent] : [],
          active_mode: 'self',
          completed_requirements: [],
          progress: sageSessionProgress(session),
          transferable_context: null,
          failed_agents: failed || [],
          failure_count: (failed || []).length,
        },
    constraints: {
      permission_mode: state.permission,
      incumbent_model: incumbentModel,
      max_team_size: 5,
    },
  };
}

function sageRoutePayload(prompt, agent, failed, session) {
  const context = sageRouteContext(session, failed);
  return {
    prompt,
    agent,
    failed: context.failed,
    // 不可用模型走独立字段：同样硬排除，但不该被算成「这个任务失败过几次」
    unavailable: modelBlockList(),
    state: context.state,
    constraints: context.constraints,
  };
}

function sageExecutor(decision, executorId) {
  const configured = decision && decision.executors && decision.executors[executorId];
  if (configured) return { id: executorId, ...configured };
  const runtime = String(executorId || '').split('::')[0];
  return {
    id: executorId,
    runtime,
    model: null,
    label: AGENTS[runtime]?.label || executorId,
    role: 'Generalist',
  };
}

function sageExecutorLabel(decision, executorId) {
  const executor = sageExecutor(decision, executorId);
  return executor.label + (executor.role ? ` · ${executor.role}` : '') +
    (executor.effort ? ` · ${executor.effort}` : '') +
    (executor.fast ? ' · Fast' : '');
}

function applySagePrimary(decision) {
  const executor = sageExecutor(decision, decision.primary);
  if (executor.runtime && executor.runtime !== state.agent) setAgent(executor.runtime);
  if (executor.model) state.model = executor.model;
  if (decision.primary_effort) state.effort = decision.primary_effort;
  syncAgentUI();
  return executor;
}

/** SAGE 决策卡片（折叠，标题展示模式与执行者） */
function sageCard(d) {
  const MODE_CN = { self: '继续当前', handoff: '移交', collaborate: '协作' };
  // 移交/协作时默认展开，让「去了哪、为什么」一眼可见
  const card = el('div', 'card sage' + (d.mode === 'self' || d.pinned ? '' : ' open'));
  const head = el('div', 'card-head');
  head.appendChild(el('span', 'card-caret', '▸'));
  const who = sageExecutorLabel(d, d.primary);
  const modeText = d.pinned
    ? (CUR_LANG === 'en' ? 'stay put' : '留在本会话')
    : d.in_place
      ? (CUR_LANG === 'en' ? 'take over in place' : '原地接管')
      : t(MODE_CN[d.mode] || d.mode);
  head.appendChild(
    el('span', 'card-title', t('🧭 SAGE 路由') + ' · ' + modeText + ' → ' + who)
  );
  const body = el('div', 'card-body');
  const en = CUR_LANG === 'en';
  const REQ_CN = {
    analysis: '分析', debugging: '调试', coding: '编码', planning: '规划',
    review: '审查', docs: '文档', refactor: '重构', vision: '视觉',
  };
  const reqName = (k) => (en ? k : REQ_CN[k] || k);
  const lines = [];
  const reqEntries = Object.entries(d.requirements || {});
  if (reqEntries.length) {
    lines.push(
      (en ? 'Task makeup: ' : '任务构成：') +
        reqEntries.map(([k, v]) => reqName(k) + ' ' + Math.round(v * 100) + '%').join('、') +
        (d.requirements_matched === false
          ? (en ? ' (no keyword matched — inferred from the incumbent\'s strengths)' : '（未命中任何关键词，按在任执行者的强项推定）')
          : '')
    );
  }
  // 判定结论（自然语言）
  if (d.mode === 'handoff' && d.pinned) {
    lines.push(
      en
        ? `Verdict: routing favours ${who}, but this session is already a handoff target — chaining further would push the result one more level away from its origin, with no end in sight. Staying put.`
        : `判定：路由倾向 ${who}，但本会话已经是移交的接管方——再往下移交只会让结论离来源又远一层，且没有终点。就留在这里继续。`
    );
  } else if (d.mode === 'handoff' && d.in_place) {
    lines.push(
      en
        ? `Verdict: ${who} is the specialist for this. Same CLI, so it takes over in place with a model switch — no new session, context stays put.`
        : `判定：这类任务 ${who} 更擅长；同一 CLI，直接在本会话换模型接管，不另开会话、上下文原地保留。`
    );
  } else if (d.mode === 'handoff') {
    lines.push(
      en
        ? `Verdict: ${who} is the specialist for this — its advantage exceeds the modeled context-transfer and switching loss.`
        : `判定：这类任务 ${who} 更擅长，其优势超过模型估算的上下文转移与切换损失，移交后由它接管。`
    );
  } else if (d.mode === 'self') {
    lines.push(
      en
        ? `Verdict: the current agent (${who}) is already the best fit — no handoff or teaming needed.`
        : `判定：当前的 ${who} 就是最合适的执行者，无需移交或组队。`
    );
  }
  const decisionPartners = d.partners || (d.partner ? [d.partner] : []);
  if (decisionPartners.length) {
    const peers = decisionPartners.map((partner) => sageExecutorLabel(d, partner)).join('、');
    lines.push(
      en
        ? `Verdict: use an auto-sized team — owner ${who} keeps ownership; ${peers} execute their assigned requirement-DAG nodes.`
        : `判定：使用自动规模团队——所有者 ${who} 保留所有权；${peers} 按需求 DAG 执行各自分配节点。`
    );
  }
  const assignmentEntries = Object.entries(d.assignments || {});
  if (assignmentEntries.length) {
    lines.push(
      (en ? 'Assignments: ' : '需求分工：') +
        assignmentEntries
          .map(([requirement, agent]) =>
            reqName(requirement) + ' → ' + sageExecutorLabel(d, agent) +
            (d.efforts && d.efforts[requirement] ? ` · 本节点 ${d.efforts[requirement]}` : '')
          )
          .join('；')
    );
  }
  if ((d.topology || []).length) {
    lines.push(
      (en ? 'Topology: ' : '通信拓扑：') +
        d.topology
          .map(([from, to]) => sageExecutorLabel(d, from) + ' → ' + sageExecutorLabel(d, to))
          .join('；')
    );
  }
  // 打分（口语化）
  const pct = (x) => Math.round((x || 0) * 100) + '%';
  lines.push(
    en
      ? `Estimated success ${pct(d.success_probability)}, capability coverage ${pct(d.coverage)} — the highest-utility option among solo / handoff / team.`
      : `预计成功率 ${pct(d.success_probability)}，能力覆盖 ${pct(d.coverage)}——在「自己干 / 移交 / 组队」三个方案里综合得分最高。`
  );
  lines.push(
    (en ? 'Estimated: ' : '估算：') +
      `cost ${Number(d.cost || 0).toFixed(3)} · latency ${Math.round(d.latency_ms || 0)}ms · ` +
      `risk ${Number(d.risk || 0).toFixed(3)} · utility ${Number(d.utility || 0).toFixed(3)}`
  );
  lines.push(
    en
      ? `Auto team: ${d.team_size || (d.agents || []).length} / ${d.team_limit || '-'} · complexity ${d.complexity?.label || 'unknown'}`
      : `自动团队：${d.team_size || (d.agents || []).length} / 上限 ${d.team_limit || '-'} · 复杂度 ${{ simple: '简单', moderate: '中等', complex: '复杂', very_complex: '很复杂' }[d.complexity?.label] || '未知'} · 汇总 ${d.summary_effort || '默认'}`
  );
  lines.push(
    en
      ? 'Schedule: requirements assigned to the same executor run serially; independent requirements assigned to different executors run in parallel.'
      : '调度规则：同一 executor 的需求串行执行；不同 executor 的无依赖需求并行执行。'
  );
  const pre = el('pre', 'io-pre');
  pre.textContent = lines.join('\n');
  if (d.explanation) pre.title = d.explanation; // 算法原始解释放悬浮提示
  body.appendChild(pre);
  card.appendChild(head);
  card.appendChild(body);
  head.addEventListener('click', () => card.classList.toggle('open'));
  return card;
}

function permissionForAgent(agent) {
  const supported = new Set((AGENTS[agent] && AGENTS[agent].permissions || []).map((p) => p.value));
  if (supported.has(state.permission)) return state.permission;
  if (state.permission === 'read-only') return supported.has('plan') ? 'plan' : 'default';
  if (state.permission === 'plan') return supported.has('read-only') ? 'read-only' : 'default';
  return supported.has('default') ? 'default' : [...supported][0];
}

function sageRequirementPrompt(
  decision, requirement, originalTask, dependencyOutputs, workflowId, primaryRef, sourceContext
) {
  const owner = decision.primary;
  const agent = decision.assignments[requirement];
  const assignments = Object.entries(decision.assignments || {})
    .map(([name, assignee]) => `${name} → ${sageExecutorLabel(decision, assignee)}`)
    .join('；');
  const topology = (decision.topology || [])
    .map(([from, to]) => `${sageExecutorLabel(decision, from)} → ${sageExecutorLabel(decision, to)}`)
    .join('；');
  const dependencies = (decision.dependencies && decision.dependencies[requirement]) || [];
  const prior = dependencyOutputs.length
    ? '\n\n依赖节点产出：\n' + dependencyOutputs.join('\n\n')
    : '';
  return (
    `【SAGE COLLABORATE · ${requirement}】\n` +
    `协作标识：${workflowId}\n` +
    (primaryRef ? `主会话：${primaryRef}\n` : '') +
    `任务所有者：${sageExecutorLabel(decision, owner)}\n` +
    `当前执行者：${sageExecutorLabel(decision, agent)}\n` +
    `你只负责需求节点：${requirement}。不要代做其他 Agent 名下的需求。\n` +
    `本节点思考强度：${decision.efforts && decision.efforts[requirement] || '默认'}\n` +
    `完整分工：${assignments || '无'}\n` +
    `通信拓扑：${topology || '无跨 Agent 边'}\n` +
    `本节点依赖：${dependencies.join('、') || '无'}\n\n` +
    `原始任务：\n${originalTask}` + (sourceContext || '') + prior +
    '\n\n请完成本节点并给出可供下游节点直接使用的明确产出。'
  );
}

function sageSummaryPrompt(decision, originalTask, results, workflowId, primaryRef) {
  const outputs = Object.entries(results)
    .map(([name, result]) => {
      const agent = decision.assignments[name];
      const who = sageExecutorLabel(decision, agent);
      const text = result.finalText ? result.finalText.slice(-6000) : result.error || '无产出';
      return `【${name} · ${who} · ${result.ok ? '成功' : '失败'}】\n${text}`;
    })
    .join('\n\n');
  return (
    '【SAGE COLLABORATE · 所有者汇总】\n' +
    `协作标识：${workflowId}\n` +
    (primaryRef ? `主会话：${primaryRef}\n` : '') +
    `任务所有者：${sageExecutorLabel(decision, decision.primary)}\n` +
    `当前执行者：${sageExecutorLabel(decision, decision.primary)}\n` +
    '你是任务所有者。' +
    '请依据官方 assignments/DAG 的各节点产出，核对一致性、解决冲突并给出最终结果。\n\n' +
    `原始任务：\n${originalTask}\n\n节点产出：\n${outputs || '无可用节点产出'}`
  );
}

/* ---------- 跨会话上下文转移 ---------- */

const SAGE_CONTEXT_BUDGET = 12000; // 附带的来源记录字符上限
const SAGE_TURN_MAX = 3000; // 单轮截断长度

/** 被移交 / 被拉进协作的 agent 拿到的是全新会话。只发一句原始任务时，
 *  「生成文档」这类追问它无从判断对象，只能在工作区里另找题目。
 *  官方 HANDOFF 的效用估算本就假设上下文可转移（transferable_context=1），
 *  这里把实际转移补齐：附上来源会话的用户提问与每轮最终回答。 */
async function sageSourceContext(sess, sinceTs) {
  if (!sess || !sess.id) return '';
  try {
    return await buildSageSourceContext(sess, sinceTs || 0);
  } catch (_) {
    return ''; // 取不到或解析异常都退回无上下文，绝不阻塞本轮执行
  }
}

/** sinceTs > 0 表示这条分支已经转过一次上下文，只补它之后的新增轮次。 */
async function buildSageSourceContext(sess, sinceTs) {
  const tr = await api.get(
    '/api/session?' +
      new URLSearchParams({ agent: sess.agent, id: sess.id, project: sess.project || '' })
  );
  const msgs = tr.messages || [];
  const finals = turnFinalAssistants(msgs);
  const who = AGENTS[sess.agent] ? AGENTS[sess.agent].label : sess.agent;
  const turns = [];
  msgs.forEach((m, i) => {
    if (sinceTs) {
      const ts = Date.parse(m.ts || '');
      if (!Number.isFinite(ts) || ts <= sinceTs) return;
    }
    const text = (m.blocks || [])
      .filter((b) => b.kind === 'text' && b.text)
      .map((b) => b.text)
      .join('\n')
      .trim();
    if (!text) return;
    if (m.role === 'user') {
      const shown = visibleUserText(text);
      if (shown) turns.push('〔用户〕' + shown);
    } else if (m.role === 'assistant' && finals.has(i)) {
      turns.push(`〔${who}〕` + text);
    }
  });
  if (!turns.length) return '';
  // 预算内从最近一轮往回收，保证紧邻当前任务的上下文一定在。
  // 这里不能用 snippet()：它会把换行压成空格，长回答的结构会整段糊掉。
  const clip = (text) =>
    text.length > SAGE_TURN_MAX ? text.slice(0, SAGE_TURN_MAX) + '\n…（本轮已截断）' : text;
  const kept = [];
  let used = 0;
  for (let i = turns.length - 1; i >= 0; i--) {
    const item = clip(turns[i]);
    if (kept.length && used + item.length > SAGE_CONTEXT_BUDGET) break;
    kept.unshift(item);
    used += item.length;
  }
  const omitted = turns.length - kept.length;
  return (
    SageScheduler.SAGE_CONTEXT_MARKER +
    (sinceTs
      ? `以下是来源会话（${who}）自上次移交以来的新增记录`
      : `以下是来源会话（${who}）在移交前的对话记录`) +
    (omitted > 0 ? `（较早的 ${omitted} 段已省略）` : '') +
    '。\n当前任务要放在这段上下文里理解，不要在工作区里另找题目；' +
    '记录不足以确定任务对象时，先说明缺什么，不要凭猜测产出。\n\n' +
    kept.join('\n\n')
  );
}

/** 被依赖阻塞或 DAG 成环而没跑的节点也要在正文留痕，
 *  否则它只在汇总里以「失败」出现，用户在页面上完全找不到对应节点。 */
function appendSageSkipped(decision, requirement, reason) {
  const agent = (decision.assignments || {})[requirement];
  const who = agent ? ' · ' + sageExecutorLabel(decision, agent) : '';
  chatMsgs.appendChild(
    renderDivider(
      `⏭ SAGE ${CUR_LANG === 'en' ? 'node' : '节点'} · ${requirement}${who} · ` +
        (CUR_LANG === 'en' ? 'skipped' : '未执行')
    )
  );
  chatMsgs.appendChild(el('div', 'status-line', '⏭ ' + reason));
}

/* ---------- SAGE 跨子会话用量聚合（顶部用量条在协作期间照常走字） ---------- */

/** 每个节点各自记账（codex 会发整场绝对值，claude 发增量），总量 = 各节点求和。
 *  上下文占比只取所有者节点——跨会话的 context 相加没有意义。 */
function beginSageUsage() {
  sageUsage = {
    parts: new Map(),
    usage: { input: 0, output: 0, cr: 0, cw: 0, ctx: 0, window: 0, has: false, abs: false },
    base: null,
    spd0: null,
    startedAt: Date.now(),
    timer: setInterval(renderUsageBar, 1000),
  };
}

function endSageUsage() {
  if (!sageUsage) return;
  clearInterval(sageUsage.timer);
  renderUsageBar(); // 定格最终数值
  sageUsage = null;
}

const SAGE_USAGE_FIELDS = [['input', 'input'], ['output', 'output'],
  ['cr', 'cache_read'], ['cw', 'cache_write']];

function sageUsageApply(key, ev, isOwner) {
  if (!sageUsage) return;
  let part = sageUsage.parts.get(key);
  if (!part) {
    part = { input: 0, output: 0, cr: 0, cw: 0 };
    sageUsage.parts.set(key, part);
  }
  for (const [field, evKey] of SAGE_USAGE_FIELDS) {
    const v = ev[evKey] || 0;
    part[field] = ev.mode === 'set' ? v : part[field] + v;
  }
  const u = sageUsage.usage;
  u.input = u.output = u.cr = u.cw = 0;
  for (const p of sageUsage.parts.values()) {
    u.input += p.input;
    u.output += p.output;
    u.cr += p.cr;
    u.cw += p.cw;
  }
  u.has = true;
  if (isOwner) {
    if (ev.context) u.ctx = ev.context;
    if (ev.window) u.window = ev.window;
  }
  try {
    renderUsageBar(); // 展示层异常绝不打断流式读取
  } catch (_) { /* 忽略 */ }
}

/** 独立 SAGE 节点流：每个并行阶段拥有自己的 DOM、事件状态和会话 id。 */
async function runSageStage(options) {
  const { requirement, prompt, project, sessionId, isOwner, signal, onSession } = options;
  const executor = options.executor;
  const agent = executor.id;
  const runtime = executor.runtime;
  const initialSessionId = sessionId || null;
  const stageEffort = options.effort || executor.effort || null;
  const label = executor.label + (executor.role ? ` · ${executor.role}` : '') +
    (stageEffort ? ` · ${stageEffort}` : '');
  const executorPermission = (executor.permissions || []).includes('workspace_write')
    ? permissionForAgent(runtime)
    : runtime === 'codex' ? 'read-only' : 'plan';
  chatMsgs.appendChild(
    renderDivider(
      requirement === '__summary__'
        ? `🤝 SAGE 汇总 · ${label}`
        : requirement === '__handoff__'
          ? `🧭 SAGE 移交 · ${label}`
        : `🤝 SAGE 节点 · ${requirement} · ${label}`
    )
  );
  const bodyEl = el('div', 'msg-asst streaming');
  chatMsgs.appendChild(bodyEl);
  const ctx = { bodyEl, lastTool: null, lastGroup: null, planCard: null, fileEdits: new Set() };
  // 活动条按节点独立：并行波次里多个节点同时跑，共用一条会互相抢占
  const stage = { ctx, ticker: null, tickerData: null, tickerOpen: false };
  const usageKey = `${executor.id}:${requirement}`;
  const startedAt = Date.now();
  let sid = sessionId || null;
  let finalText = '';
  let doneOk = false;
  let error = '';
  let textEl = null;
  const stderrLines = [];
  let thinking = null;
  let outputTokens = 0;
  const ensureText = () => {
    if (!textEl) {
      textEl = el('div', 'md');
      bodyEl.appendChild(textEl);
    }
    return textEl;
  };
  const appendText = (text) => {
    finalText = (finalText + text).slice(-24000);
    renderMarkdown(ensureText(), finalText);
    bodyEl.appendChild(textEl);
  };
  const handler = (ev) => {
    if (!ev || typeof ev !== 'object') return;
    // 与主路径一致：只有本来就贴着底部时才跟随滚动，否则用户翻不上去
    const near = chatNearBottom();
    switch (ev.t) {
      case 'run':
        if (ev.run_id) state.runIds.add(ev.run_id);
        break;
      case 'init':
        if (ev.session_id) {
          sid = ev.session_id;
          const visibleOwner = !!(
            isOwner && state.session &&
            state.session.agent === runtime &&
            clientNorm(state.session.project || '') === clientNorm(project || '') &&
            (initialSessionId ? state.session.id === initialSessionId : !state.session.id)
          );
          if (visibleOwner && state.session.id !== sid) {
            state.session.id = sid;
            prependConvRow();
            renderProjects();
          }
          if (onSession) onSession(agent, sid);
          if (
            visibleOwner && state.pendingSage && state.session && state.session.id === sid
          ) {
            sageStoreSave(state.session.agent + ':' + sid, state.pendingSage);
            state.pendingSage = null;
          }
        }
        break;
      case 'delta':
        if (!ev.text) break;
        removeTicker(stage); // 正文/思考流本身就是可见进度
        endToolGroup(ctx);
        if (ev.channel === 'thinking') {
          if (!thinking) {
            thinking = thinkingCard('', true);
            thinking.raw = '';
            bodyEl.appendChild(thinking);
          }
          thinking.raw += ev.text || '';
          thinking.querySelector('.think-body').textContent = thinking.raw;
        } else {
          appendText(ev.text || '');
        }
        break;
      case 'text':
        removeTicker(stage);
        endToolGroup(ctx);
        appendText((finalText ? '\n' : '') + (ev.text || ''));
        break;
      case 'thinking':
        removeTicker(stage);
        endToolGroup(ctx);
        bodyEl.appendChild(thinkingCard(ev.text || '', false));
        break;
      case 'plan':
        upsertPlan(ctx, ev.items || []);
        break;
      case 'tool_use':
        appendToolUse(ctx, ev.name || '工具', ev.text || '', ev.id);
        updateTicker(tickerLabel(ev.name || ''), toolDetail(ev.text || ''), stage);
        break;
      case 'tool_result':
        appendToolResult(ctx, ev.text || '');
        break;
      case 'file_edit':
        if (ev.path) {
          ctx.fileEdits.add(ev.path);
          updateTicker(CUR_LANG === 'en' ? 'Editing' : '正在编辑', ev.path, stage);
        }
        break;
      // 子 agent 过程嵌进触发它的工具卡片，与主路径一致
      case 'sub_text':
        appendSubEvent(ev.sub, 'text', '', ev.text || '');
        break;
      case 'sub_tool':
        appendSubEvent(ev.sub, 'tool', ev.name || '工具', ev.text || '');
        break;
      case 'usage':
        outputTokens = ev.mode === 'set' ? ev.output || outputTokens : outputTokens + (ev.output || 0);
        sageUsageApply(usageKey, ev, !!isOwner);
        break;
      case 'status':
        bodyEl.appendChild(el('div', 'status-line', '⏳ ' + (ev.text || '')));
        break;
      case 'stderr':
        if (ev.text) stderrLines.push(ev.text);
        break;
      case 'done':
        if (ev.session_id) sid = ev.session_id;
        doneOk = !!ev.ok;
        error = ev.error || '';
        break;
      default:
        break;
    }
    if (near) scrollChat();
  };
  try {
    await streamChat(
      {
        agent: runtime,
        project,
        prompt,
        session_id: sid,
        model: executor.model || null,
        permission: executorPermission,
        effort: stageEffort,
        fast: fastForAgent(runtime),
        memory: state.memOn,
      },
      handler,
      signal
    );
  } catch (err) {
    error = err && err.name === 'AbortError' ? '已停止' : (err && err.message) || String(err);
  }
  if (thinking) thinking.classList.remove('open');
  removeTicker(stage);
  endToolGroup(ctx);
  flushFilesCard(ctx);
  bodyEl.classList.remove('streaming');
  const importantStderr = stderrLines.filter((line) => !SageScheduler.isBenignStderr(line));
  if (importantStderr.length) {
    bodyEl.appendChild(
      stderrCard(doneOk ? '⚠ 运行提示' : '⚠ 失败详情', importantStderr)
    );
  }
  if (!doneOk) {
    bodyEl.appendChild(
      el('div', error === '已停止' ? 'status-line' : 'error-bar', error || '节点执行失败')
    );
  } else if (Date.now() - startedAt > 3000) {
    bodyEl.appendChild(el('div', 'done-line', t('已处理') + ' ' + fmtDuration(Date.now() - startedAt)));
  }
  if (finalText) appendMsgActions(bodyEl, finalText.trim());
  return {
    agent,
    runtime,
    model: executor.model || null,
    effort: stageEffort,
    requirement,
    sessionId: sid,
    ok: doneOk,
    error,
    finalText,
    outputTokens,
    latencyMs: Date.now() - startedAt,
    executed: true,
  };
}

/** 官方动态团队 COLLABORATE 执行器：按 requirement DAG 分波次，跨 executor 并行。 */
async function runSageCollaboration(decision, originalTask) {
  const owner = decision.primary;
  const ownerExecutor = sageExecutor(decision, owner);
  const collaborationProject = state.session.project;
  const partners = decision.partners || (decision.agents || []).filter((agent) => agent !== owner);
  const assignments = decision.assignments || {};
  const dependencies = decision.dependencies || {};
  const requirementNames = Object.keys(assignments);
  const pending = new Set(requirementNames);
  const results = {};
  const workflowId =
    decision.workflow_id || `sage-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  decision.workflow_id = workflowId;
  const sessions = { [owner]: state.session && state.session.id || null };
  if (state.session && state.session.id) {
    const links = collabStoreLoad().links[ownerExecutor.runtime + ':' + state.session.id] || [];
    for (const partner of partners) {
      const partnerExecutor = sageExecutor(decision, partner);
      const existing = [...links].reverse().find((link) =>
        link.executor === partner ||
        (!link.executor && link.partner.startsWith(partnerExecutor.runtime + ':'))
      );
      if (existing) sessions[partner] = existing.partner.slice(existing.partner.indexOf(':') + 1);
    }
  }
  const startedAt = Date.now();
  const controller = new AbortController();
  state.streaming = true;
  state.runId = null;
  state.runIds.clear();
  state.abort = controller;
  setSendButton(true);
  state.pendingSage = decision;
  const linkedPartners = new Map();
  const maybeLinkPartners = () => {
    const ownerSessionId = sessions[owner];
    if (!ownerSessionId) return;
    const primaryKey = ownerExecutor.runtime + ':' + ownerSessionId;
    for (const partner of partners) {
      if (!sessions[partner] || linkedPartners.get(partner) === sessions[partner]) continue;
      const executor = sageExecutor(decision, partner);
      const cats = requirementNames.filter((name) => assignments[name] === partner);
      linkedPartners.set(partner, sessions[partner]);
      collabLinkSave(
        primaryKey,
        {
          partner: executor.runtime + ':' + sessions[partner],
          executor: partner,
          model: executor.model,
          effort: executor.effort || null,
          label: sageExecutorLabel(decision, partner),
          kind: 'pipeline',
          cats: cats.join('、'),
          workflow_id: workflowId, // 有 id = 官方 DAG，收尾由 __summary__ 负责，不再走旧版回注
          ts: Date.now(),
        },
        executor.runtime + ':' + sessions[partner]
      );
    }
    renderCollabPanel();
  };
  const onSession = (agent, sid) => {
    sessions[agent] = sid;
    if (agent === owner) {
      sageStoreSave(ownerExecutor.runtime + ':' + sid, decision);
      state.pendingSage = null;
    }
    maybeLinkPartners();
  };
  beginSageUsage();
  // 所有者节点跑在既有会话里、本就带着历史；搭档是全新会话，得把来源记录带过去
  const sourceContext = await sageSourceContext(
    state.session && state.session.id ? state.session : null
  );
  try {
    while (pending.size && !controller.signal.aborted) {
      const wave = SageScheduler.nextWave(pending, results, dependencies, assignments);
      if (wave.cycle) {
        for (const name of pending) {
          results[name] = {
            agent: assignments[name], requirement: name, ok: false, executed: false,
            finalText: '', error: '需求 DAG 无可执行节点', latencyMs: 0,
          };
          appendSageSkipped(decision, name, '需求 DAG 无可执行节点');
        }
        pending.clear();
        break;
      }
      for (const blocked of wave.blocked) {
        results[blocked.name] = {
          agent: assignments[blocked.name], requirement: blocked.name, ok: false, executed: false,
          finalText: '', error: `依赖节点 ${blocked.failedDependency} 失败`, latencyMs: 0,
        };
        pending.delete(blocked.name);
        appendSageSkipped(
          decision, blocked.name,
          (CUR_LANG === 'en' ? 'dependency failed: ' : '依赖节点失败：') + blocked.failedDependency
        );
      }
      await SageScheduler.executeWave(wave.groups, async (agent, name) => {
        if (controller.signal.aborted) return;
        const dependencyOutputs = (dependencies[name] || []).map((dependency) => {
          const result = results[dependency];
          return `【${dependency}】\n${result.finalText || result.error || '无产出'}`;
        });
        const result = await runSageStage({
          executor: sageExecutor(decision, agent),
          requirement: name,
          effort: decision.efforts && decision.efforts[name],
          prompt: sageRequirementPrompt(
            decision,
            name,
            originalTask,
            dependencyOutputs,
            workflowId,
            sessions[owner] ? `${ownerExecutor.runtime}:${sessions[owner]}` : '',
            agent === owner ? '' : sourceContext
          ),
          project: collaborationProject,
          sessionId: sessions[agent],
          isOwner: agent === owner,
          signal: controller.signal,
          onSession,
        });
        sessions[agent] = result.sessionId;
        results[name] = result;
        pending.delete(name);
        maybeLinkPartners();
      });
    }
    if (controller.signal.aborted) return;
    const summary = await runSageStage({
      executor: sageExecutor(decision, owner),
      requirement: '__summary__',
      effort: decision.summary_effort,
      prompt: sageSummaryPrompt(
        decision,
        originalTask,
        results,
        workflowId,
        sessions[owner] ? `${ownerExecutor.runtime}:${sessions[owner]}` : ''
      ),
      project: collaborationProject,
      sessionId: sessions[owner],
      isOwner: true,
      signal: controller.signal,
      onSession,
    });
    sessions[owner] = summary.sessionId;
    maybeLinkPartners();
    const requirementScores = {};
    let weighted = 0;
    let totalWeight = 0;
    for (const name of requirementNames) {
      const score = results[name] && results[name].ok ? 1 : 0;
      const weight = Number((decision.requirements || {})[name] || 0);
      requirementScores[name] = score;
      weighted += score * weight;
      totalWeight += weight;
    }
    const agentScores = {};
    for (const agent of new Set(Object.values(assignments))) {
      const owned = requirementNames.filter((name) => assignments[name] === agent && results[name]?.executed);
      if (owned.length) {
        agentScores[agent] = owned.reduce((sum, name) => sum + (results[name].ok ? 1 : 0), 0) / owned.length;
      }
    }
    // 逐节点记失败：模型级不可用的执行体要立刻退出后续路由候选
    for (const name of requirementNames) {
      const node = results[name];
      if (node && node.executed && !node.ok) sageRememberFailure(assignments[name], node.error);
    }
    if (!summary.ok) sageRememberFailure(owner, summary.error);
    await api.post('/api/sage/outcome', {
      decision_blob: decision.decision_blob,
      success: totalWeight > 0 ? weighted / totalWeight : summary.ok ? 1 : 0,
      actual_latency_ms: Date.now() - startedAt,
      agent_scores: agentScores,
      requirement_scores: requirementScores,
    });
  } catch (error) {
    chatMsgs.appendChild(el('div', 'error-bar', 'SAGE 协作执行失败：' + (error.message || error)));
  } finally {
    endSageUsage();
    state.streaming = false;
    state.abort = null;
    state.runId = null;
    state.runIds.clear();
    state.pendingSage = null;
    setSendButton(false);
    if (state.stopRequested) state.stopRequested = false;
    if (state.session) promptInput.placeholder = t('继续这个会话…');
    promptInput.focus();
    loadConvs();
    loadProjects();
  }
}

function existingHandoffLink(previous, decision, executor) {
  if (!previous || !previous.id) return null;
  const links = collabStoreLoad().links[previous.agent + ':' + previous.id] || [];
  return SageScheduler.reusableHandoffLink(links, executor.runtime);
}

/** 官方 HANDOFF：peer 接管所有权。首次移交新建会话，后续追问接回同一条分支。
 *  只处理跨 runtime 的移交——同 runtime 已在 sendPrompt 降级为原地接管（不另开会话）。 */
async function runSageHandoff(decision, originalTask) {
  const previous = state.session;
  const executor = applySagePrimary(decision);
  if (state.agentFilter && state.agentFilter !== executor.runtime) setAgentFilter('');
  const reusedLink = existingHandoffLink(previous, decision, executor);
  const reusedId = reusedLink
    ? reusedLink.partner.slice(reusedLink.partner.indexOf(':') + 1)
    : null;
  state.session = {
    agent: executor.runtime,
    id: reusedId,
    project: previous.project,
    // 复用时沿用分支自己的标题，别被本轮追问覆盖
    title: (reusedLink && reusedLink.title) || snippet(originalTask, 40),
    model: executor.model || null,
    effort: decision.primary_effort || executor.effort || null,
  };
  setChatHead(state.session);
  setActiveRow(reusedId ? executor.runtime + ':' + reusedId : null);
  state.pendingSage = decision;
  state.streaming = true;
  state.runId = null;
  state.runIds.clear();
  const controller = new AbortController();
  state.abort = controller;
  setSendButton(true);
  const startedAt = Date.now();
  const workflowId =
    decision.workflow_id || `sage-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  decision.workflow_id = workflowId;
  beginSageUsage();
  // 移交前先把来源会话的记录取出来，否则接管方只拿到一句孤立的任务。
  // 复用分支时只补上次移交之后的新增轮次，不重复灌整段历史。
  const contextTs = Date.now();
  const sourceContext = await sageSourceContext(
    previous,
    reusedLink ? reusedLink.context_ts || 0 : 0
  );
  try {
    const result = await runSageStage({
      executor: sageExecutor(decision, decision.primary),
      requirement: '__handoff__',
      effort: decision.primary_effort,
      prompt:
        '【SAGE HANDOFF】路由判定你接管本任务的完整所有权。请独立完成并给出最终结果。\n' +
        `协作标识：${workflowId}\n` +
        (previous.id ? `来源会话：${previous.agent}:${previous.id}\n` : '') +
        `任务所有者：${sageExecutorLabel(decision, decision.primary)}\n` +
        `当前执行者：${sageExecutorLabel(decision, decision.primary)}\n\n` +
        originalTask + sourceContext,
      project: state.session.project,
      sessionId: reusedId,
      isOwner: true,
      signal: controller.signal,
      onSession: (_, sid) => {
        const targetKey = executor.runtime + ':' + sid;
        sageStoreSave(targetKey, decision);
        if (previous.id) {
          collabLinkSave(
            previous.agent + ':' + previous.id,
            {
              partner: targetKey,
              executor: decision.primary,
              model: executor.model || null,
              effort: decision.primary_effort || executor.effort || null,
              label: sageExecutorLabel(decision, decision.primary),
              kind: 'handoff',
              cats: '',
              workflow_id: workflowId,
              // 分支标题与上下文水位：后续追问据此复用会话、只补增量
              title: state.session.title,
              context_ts: contextTs,
              ts: Date.now(),
            },
            targetKey
          );
          if (
            state.session && state.session.agent === executor.runtime && state.session.id === sid
          ) {
            syncBackPrimaryUI(state.session);
          }
          loadConvs();
          loadProjects();
          renderProjects();
        }
      },
    });
    if (controller.signal.aborted) return;
    if (result.ok) state.sageFailed = null;
    else sageRememberFailure(decision.primary, result.error);
    const requirementScores = Object.fromEntries(
      Object.keys(decision.assignments || {}).map((name) => [name, result.ok ? 1 : 0])
    );
    await api.post('/api/sage/outcome', {
      decision_blob: decision.decision_blob,
      success: result.ok ? 1 : 0,
      actual_latency_ms: Date.now() - startedAt,
      agent_scores: { [decision.primary]: result.ok ? 1 : 0 },
      requirement_scores: requirementScores,
    });
  } catch (error) {
    chatMsgs.appendChild(el('div', 'error-bar', 'SAGE 移交失败：' + (error.message || error)));
  } finally {
    endSageUsage();
    state.streaming = false;
    state.abort = null;
    state.runId = null;
    state.runIds.clear();
    state.pendingSage = null;
    setSendButton(false);
    if (state.stopRequested) state.stopRequested = false;
    if (state.session) promptInput.placeholder = t('继续这个会话…');
    promptInput.focus();
    loadConvs();
    loadProjects();
  }
}

/** 协作收尾通用段：把文本以消息回注主会话，由主 agent 续跑一轮 */
async function runPrimaryFollowup(primarySess, dividerText, prompt) {
  chatMsgs.appendChild(renderDivider(dividerText));
  appendUserBubble(chatMsgs, prompt, []);
  scrollChat();
  beginAssistant();
  const req = {
    agent: primarySess.agent,
    project: primarySess.project,
    prompt,
    session_id: primarySess.id,
    model: null,
    permission: state.permission,
    effort: null,
    fast: fastForAgent(primarySess.agent),
    memory: state.memOn,
  };
  state.streaming = true;
  state.runId = null;
  setSendButton(true);
  const ac = new AbortController();
  state.abort = ac;
  try {
    await streamChat(req, handleEvent, ac.signal);
  } catch (e) {
    if (stream) {
      if (e && e.name === 'AbortError') {
        stream.ctx.bodyEl.appendChild(
          el('div', 'status-line', t(state.stopRequested ? '■ 已停止' : '↪ 已断开查看，任务在后台继续'))
        );
      } else {
        stream.ctx.bodyEl.appendChild(el('div', 'error-bar', t('请求失败：') + (e.message || e)));
      }
    }
  } finally {
    finalizeStream();
    state.streaming = false;
    state.abort = null;
    state.runId = null;
    state.runIds.clear();
    setSendButton(false);
    if (state.stopRequested) state.stopRequested = false;
    loadConvs();
  }
}

async function onSend() {
  if (state.streaming) return;
  state.runIds.clear();
  const text = promptInput.value.trim();
  const atts = state.attachments.slice();
  if (!text && !atts.length) return;
  const imgAtts = atts.filter((a) => a.kind !== 'text');
  const txtAtts = atts.filter((a) => a.kind === 'text');
  let routingPrompt = text;
  for (const attachment of txtAtts) {
    routingPrompt += (routingPrompt ? '\n\n' : '') + attachment.text;
  }
  for (const attachment of imgAtts) {
    routingPrompt +=
      (routingPrompt ? '\n\n' : '') +
      `图片/截图/视觉附件：${attachment.name || attachment.path || 'image'}`;
  }

  // SAGE 智能路由：仅新会话、非斜杠命令时决策执行者
  let sageInfo = null;
  if (state.sageOn && !state.session && routingPrompt.trim() && !text.startsWith('/')) {
    const btn = $('#sage-btn');
    setToggleChip(btn, t('🧭 路由中…'), true);
    state.sageTaskText = text;
    try {
      // 失败重路由：同一任务重发时，把上次失败的执行者交给 ExecutionState.failed_agents
      const failed =
        state.sageFailed && state.sageFailed.task === text ? state.sageFailed.agents : [];
      sageInfo = await api.post(
        '/api/sage',
        sageRoutePayload(routingPrompt, state.agent, failed, null)
      );
      if (sageInfo && sageInfo.primary) {
        const primaryExecutor = applySagePrimary(sageInfo);
        const who = sageExecutorLabel(sageInfo, sageInfo.primary);
        // 侧栏过滤会藏住被移交的新会话 → 自动放行到「全部」
        let extra = '';
        if (state.agentFilter && state.agentFilter !== primaryExecutor.runtime) {
          setAgentFilter('');
          extra = CUR_LANG === 'en' ? ' (sidebar filter reset to All)' : '（侧栏过滤已切回全部）';
        }
        const partners = (sageInfo.partners || []).map((id) => sageExecutorLabel(sageInfo, id));
        showToast(
          (partners.length
            ? CUR_LANG === 'en'
              ? `🧭 Auto team ${sageInfo.team_size}: ${who} owns; ${partners.join(', ')} collaborate`
              : `🧭 自动团队 ${sageInfo.team_size} 人：${who} 负责汇总；${partners.join('、')} 协作`
            : CUR_LANG === 'en'
              ? '🧭 Routed to ' + who
              : '🧭 已移交给 ' + who + ' 执行') + extra
        );
      }
    } catch (_) {
      sageInfo = null; // 路由失败回退当前 agent，不阻塞发送
    }
    syncAgentUI();
  }

  // 一个会话只路由一次：首轮定下执行者，之后的追问一律沿用，不再重新比较。
  //
  // 上游 SAGE 是「给定一个 Task 选最优配置」的路由器，一个任务路由一次；把每轮追问
  // 都当成新 Task 反复决策，会被 ~/.agenthub/sage_state.json 里的小样本学习噪声左右
  // ——实测同一批提问，空学习状态下 5/5 留任，带上仅 8 次更新的真实状态后 0/5 留任，
  // 每轮换一个模型，新执行者还得重新理解一遍上下文。
  //
  // 两个例外仍然要重新路由：
  //   1. 上一轮执行失败（官方的失败重路由语义，failed_agents 会带上失败的执行者）
  //   2. 这条会话还没被 SAGE 决策过（终端建的、刚打开智能路由的——那就是它的首轮）
  const sageRetry = !!(state.sageFailed && (state.sageFailed.agents || []).length);
  if (
    SageScheduler.shouldRouteFollowUp({
      sageOn: state.sageOn,
      session: state.session,
      text,
      hasAttachments: !!atts.length,
      decided: !!(
        state.session && state.session.id &&
        sageStoreLoad()[state.session.agent + ':' + state.session.id]
      ),
      retrying: sageRetry,
    })
  ) {
    state.sageTaskText = text;
    try {
      // 失败重试时务必带上失败的执行者，否则重路由可能又挑中刚栽过的那个
      sageInfo = await api.post(
        '/api/sage',
        sageRoutePayload(
          routingPrompt,
          state.session.agent,
          sageRetry ? state.sageFailed.agents : [],
          state.session
        )
      );
      if (sageInfo) {
        // 同一 CLI 的移交降级为原地接管：换模型继续当前会话，不再另开一条
        sageInfo.in_place = SageScheduler.handoffStaysInPlace(
          sageInfo,
          state.session,
          sageExecutor(sageInfo, sageInfo.primary).runtime
        );
        // 子会话不再往下移交：移交链每多一级，结论就离来源会话更远一层，
        // 而且没有终点（codex→claude→codex→…）。跨 CLI 时整个决策作废，原样继续。
        if (sageInfo.mode === 'handoff' && !sageInfo.in_place && state.backPrimary) {
          sageInfo.pinned = true;
        }
        const who = sageExecutorLabel(sageInfo, sageInfo.primary);
        if (sageInfo.pinned) {
          showToast(
            CUR_LANG === 'en'
              ? `🧭 Sub-session stays put — ${AGENTS[state.session.agent].label} continues`
              : `🧭 子会话不再移交 · 仍由 ${AGENTS[state.session.agent].label} 继续`
          );
        } else {
          const mode = sageInfo.in_place
            ? '原地接管'
            : { self: '继续当前', collaborate: '协作', handoff: '移交' }[sageInfo.mode] || sageInfo.mode;
          showToast(`🧭 ${mode} → ${who}`);
        }
      }
    } catch (_) {
      sageInfo = null; // 重规划失败回退当前会话，不阻塞追问
    }
  }

  if (!state.session) {
    // Hero 新会话
    if (!state.project) {
      showComposerError(t('请先选择项目目录（输入卡片下方的项目选择器）'));
      return;
    }
    const firstTxtAtt = atts.find((a) => a.kind === 'text');
    state.session = {
      agent: state.agent,
      id: null,
      project: state.project,
      title: snippet(text || (firstTxtAtt ? firstTxtAtt.name : '[图片]'), 40),
      model: state.model,
      effort: state.effort,
    };
    showChat();
    chatMsgs.textContent = '';
    setChatHead(state.session);
    setActiveRow(null);
  }
  hideComposerError();
  promptInput.value = '';
  autoGrow();
  state.attachments = [];
  renderAttachBar();
  // 气泡里长文本附件只显示摘要行（完整内容仍随 prompt 发送并落盘）
  const bubbleText =
    text +
    txtAtts
      .map((a) => '\n📄 ' + a.name + (CUR_LANG === 'en' ? ` (${a.text.length} chars)` : `（${a.text.length} 字）`))
      .join('');
  appendUserBubble(chatMsgs, bubbleText.trim(), imgAtts.map((a) => imageEl(a.path)));
  scrollChat();
  if (sageInfo) {
    const decisionBody = el('div', 'msg-asst sage-decision');
    decisionBody.appendChild(sageCard(sageInfo));
    chatMsgs.appendChild(decisionBody);
    state.pendingSage = sageInfo; // init 事件一到（几秒内）就落盘，防中途刷新丢失
  }

  // 长文本附件展开进 prompt；图片以本地路径写入，由 CLI 的图片查看/Read 工具读取
  let finalPrompt = text;
  for (const a of txtAtts) {
    finalPrompt += (finalPrompt ? '\n\n' : '') + a.text;
  }
  for (const a of imgAtts) {
    finalPrompt += (finalPrompt ? '\n\n' : '') + '请查看图片文件: ' + a.path;
  }
  finalPrompt = withTeamPreamble(finalPrompt, state.session.agent);

  if (sageInfo && sageInfo.mode === 'collaborate' && (sageInfo.partners || []).length) {
    await runSageCollaboration(sageInfo, finalPrompt);
    return;
  }
  if (sageInfo && sageInfo.mode === 'handoff' && !sageInfo.in_place && !sageInfo.pinned) {
    await runSageHandoff(sageInfo, finalPrompt);
    return;
  }
  // 原地接管：只把执行者的模型/思考档位换上，继续跑当前会话。
  // 会话上也要记一份：下一轮路由的「在任模型」取自 session.model，
  // 不更新的话 SAGE 会一直以为还在跑旧模型，来回判定移交。
  if (sageInfo && sageInfo.in_place) {
    const executor = applySagePrimary(sageInfo);
    if (executor.model) state.session.model = executor.model;
    const inPlaceEffort = sageInfo.primary_effort || executor.effort;
    if (inPlaceEffort) state.session.effort = inPlaceEffort;
    syncAgentUI(); // 思考档位按钮读的是 session.effort，得在写完之后再刷一次
  }

  beginAssistant();

  const req = {
    agent: state.session.agent,
    project: state.session.project,
    prompt: finalPrompt,
    session_id: state.session.id,
    model: state.model,
    permission: state.permission,
    // pinned：这次决策整个作废（执行者是当前会话自己），别把别家 CLI 的档位带过来
    effort:
      sageInfo && !sageInfo.pinned && sageInfo.primary_effort
        ? sageInfo.primary_effort
        : state.effort,
    fast: fastForAgent(state.session.agent),
    memory: state.memOn,
  };
  state.streaming = true;
  state.runId = null;
  setSendButton(true);
  const ac = new AbortController();
  state.abort = ac;
  let primaryFinal = '';
  let primaryOk = false;
  let primaryAborted = false;
  let primaryMs = null;
  let primaryError = '';
  try {
    await streamChat(req, handleEvent, ac.signal);
  } catch (err) {
    if (stream) {
      if (err && err.name === 'AbortError') {
        primaryAborted = true;
        stream.ctx.bodyEl.appendChild(
          el('div', 'status-line', t(state.stopRequested ? '■ 已停止' : '↪ 已断开查看，任务在后台继续'))
        );
      } else {
        stream.ctx.bodyEl.appendChild(el('div', 'error-bar', t('请求失败：') + ((err && err.message) || err)));
      }
    }
  } finally {
    if (stream) {
      primaryFinal = stream.finalText || '';
      primaryOk = !!stream.doneOk;
      primaryError = stream.doneError || '';
      primaryMs = Date.now() - stream.startedAt;
    }
    finalizeStream();
    state.streaming = false;
    state.abort = null;
    state.runId = null;
    state.runIds.clear();
    setSendButton(false);
    if (state.stopRequested) state.stopRequested = false;
    // 若期间已切回 Hero（新建会话），不要覆盖它的 placeholder
    if (state.session) promptInput.placeholder = t('继续这个会话…');
    promptInput.focus();
    // 会话文件已落盘，刷新侧栏（列表与项目计数）
    loadConvs();
    loadProjects();
    // 兜底：init 未触发保存时（极短运行等），结束时再存一次
    if (sageInfo && state.session && state.session.id) {
      sageStoreSave(state.session.agent + ':' + state.session.id, sageInfo);
    }
    state.pendingSage = null;
  }
  // SELF / HANDOFF 结果回喂；COLLABORATE 已由官方 DAG 执行器统一回喂。
  // pinned 的决策没被执行，回喂等于把功劳/锅算到一个没上场的执行者头上。
  if (sageInfo && sageInfo.decision_blob && !primaryAborted && !sageInfo.pinned) {
    if (primaryOk) {
      state.sageFailed = null;
    } else {
      sageRememberFailure(sageInfo.primary || req.agent, primaryError);
    }
    const requirementScores = Object.fromEntries(
      Object.keys(sageInfo.assignments || {}).map((name) => [name, primaryOk ? 1 : 0])
    );
    api
      .post('/api/sage/outcome', {
        decision_blob: sageInfo.decision_blob,
        success: primaryOk ? 1 : 0,
        actual_latency_ms: primaryMs,
        agent_scores: { [sageInfo.primary]: primaryOk ? 1 : 0 },
        requirement_scores: requirementScores,
      })
      .catch(() => {});
  }
}

/* ---------- 输入框 ---------- */

function autoGrow() {
  promptInput.style.height = 'auto';
  promptInput.style.height = Math.min(promptInput.scrollHeight, 240) + 'px';
  renderPromptHl();
}

/* ---------- 技能 token 高亮（背景层，不同技能不同颜色） ---------- */

const SKILL_PALETTE = [
  ['#e8964a', 'rgba(232,150,74,0.18)'],  // 橙
  ['#4a90e8', 'rgba(74,144,232,0.18)'],  // 蓝
  ['#9a6ee8', 'rgba(154,110,232,0.18)'], // 紫
  ['#4ab87a', 'rgba(74,184,122,0.18)'],  // 绿
  ['#e85a7a', 'rgba(232,90,122,0.18)'],  // 玫红
  ['#d8c84a', 'rgba(216,200,74,0.16)'],  // 黄
  ['#5ac8d8', 'rgba(90,200,216,0.18)'],  // 青
  ['#c88a5a', 'rgba(200,138,90,0.18)'],  // 棕
];

/** 技能名哈希 → 稳定配色（同名同色，不同技能不同色） */
function skillColor(token) {
  let h = 0;
  for (const c of token) h = (h * 31 + c.codePointAt(0)) >>> 0;
  return SKILL_PALETTE[h % SKILL_PALETTE.length];
}

/** 输入框开头的 /技能 或 $技能 token 加彩色药丸背景（文字仍由 textarea 渲染） */
function renderPromptHl() {
  const hl = $('#prompt-hl');
  if (!hl) return;
  hl.textContent = '';
  const v = promptInput.value;
  const m = v.match(/^([/$][A-Za-z0-9:_.\-]+)([\s\S]*)$/);
  if (!m) {
    hl.appendChild(document.createTextNode(v));
    return;
  }
  const [fg, bg] = skillColor(m[1]);
  const pill = el('span', 'hl-skill', m[1]);
  pill.style.background = bg;
  pill.style.boxShadow = '0 0 0 1px ' + fg + '55';
  hl.appendChild(pill);
  hl.appendChild(document.createTextNode(m[2]));
  hl.scrollTop = promptInput.scrollTop;
}

/* ---------- 事件绑定与初始化 ---------- */

function bindEvents() {
  // 输入卡片
  promptInput.addEventListener('input', (e) => {
    autoGrow();
    // 仅在「刚键入 /」时弹技能面板；删除退回到 / 不再误触发
    if (e.inputType === 'insertText' && e.data === '/' && promptInput.value === '/') {
      openSkillPicker($('#skill-btn'), true);
    }
  });
  promptInput.addEventListener('scroll', () => {
    const hl = $('#prompt-hl');
    if (hl) hl.scrollTop = promptInput.scrollTop;
  });
  // 粘贴图片 → 上传为附件；粘贴超长文本 → 折叠为文本附件卡（不灌进输入框）
  promptInput.addEventListener('paste', (e) => {
    const items = e.clipboardData && e.clipboardData.items;
    if (items) {
      for (const it of items) {
        if (it.type && it.type.startsWith('image/')) {
          e.preventDefault();
          const f = it.getAsFile();
          if (f) addAttachment(f);
          return;
        }
      }
    }
    const txt = e.clipboardData ? e.clipboardData.getData('text/plain') : '';
    if (txt && (txt.length > 800 || txt.split('\n').length > 10)) {
      e.preventDefault();
      const first = (txt.trim().split('\n')[0] || '').trim();
      state.attachments.push({
        kind: 'text',
        text: txt,
        name: snippet(first || (CUR_LANG === 'en' ? 'Pasted text' : '粘贴文本'), 26),
      });
      renderAttachBar();
    }
  });
  $('#attach-btn').addEventListener('click', () => $('#file-input').click());
  $('#file-input').addEventListener('change', (e) => {
    for (const f of e.target.files) {
      if (f.type.startsWith('image/')) addAttachment(f);
    }
    e.target.value = '';
  });
  promptInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey && !e.isComposing && e.keyCode !== 229) {
      e.preventDefault();
      if (!state.streaming) onSend();
    }
  });
  $('#send-btn').addEventListener('click', () => {
    if (state.streaming) stopRun();
    else onSend();
  });

  // agent 切换（Hero 标题 + 输入卡片徽标）
  const agentMenuItems = () => [
    { value: 'claude', label: 'Claude Code', checked: state.agent === 'claude' },
    { value: 'codex', label: 'Codex', checked: state.agent === 'codex' },
  ];
  $('#agent-switch').addEventListener('click', (e) => {
    e.stopPropagation();
    showMenu(e.currentTarget, agentMenuItems(), (it) => setAgent(it.value));
  });
  $('#composer-agent').addEventListener('click', (e) => {
    e.stopPropagation();
    if (!canSwitchAgent()) return; // 会话已绑定 agent
    showMenu(e.currentTarget, agentMenuItems(), (it) => setAgent(it.value));
  });

  // Claude 可手动切 fastMode；Codex 所有模型固定使用 service_tier=fast。
  $('#fast-btn').addEventListener('click', () => {
    if (currentAgent() === 'codex') {
      showToast(CUR_LANG === 'en'
        ? 'Fast mode is always enabled for every Codex model'
        : 'Codex 所有模型固定使用 Fast 模式');
      return;
    }
    state.fast = !state.fast;
    syncAgentUI();
    savePrefs();
  });

  // SAGE 智能路由开关
  $('#sage-btn').addEventListener('click', () => {
    state.sageOn = !state.sageOn;
    if (state.sageOn && state.teamOn) {
      state.teamOn = false;
      localStorage.setItem('ah-team', '0');
      showToast(CUR_LANG === 'en' ? 'Team mode turned off: it cannot be combined with smart routing' : '已关闭团队模式：不能与智能路由同时开启');
    }
    localStorage.setItem('ah-sage', state.sageOn ? '1' : '0');
    syncAgentUI();
  });

  // OpenViking 记忆开关
  $('#mem-btn').addEventListener('click', () => {
    state.memOn = !state.memOn;
    localStorage.setItem('ah-mem', state.memOn ? '1' : '0');
    syncAgentUI();
  });

  // 团队模式：点击开关；已开启时再点在 2..5 间循环队员数，到顶关闭
  $('#team-btn').addEventListener('click', () => {
    if (!state.teamOn) {
      state.teamOn = true;
      if (state.sageOn) {
        state.sageOn = false;
        localStorage.setItem('ah-sage', '0');
        showToast(CUR_LANG === 'en' ? 'Smart routing turned off: it cannot be combined with team mode' : '已关闭智能路由：不能与团队模式同时开启');
      }
    } else if (state.teamSize < 5) {
      state.teamSize++;
    } else {
      state.teamOn = false;
      state.teamSize = 3;
    }
    localStorage.setItem('ah-team', state.teamOn ? '1' : '0');
    localStorage.setItem('ah-team-n', String(state.teamSize));
    syncAgentUI();
  });

  // 技能选择器（按钮 / 输入框开头键入斜杠）
  $('#skill-btn').addEventListener('click', (e) => {
    e.stopPropagation();
    openSkillPicker(e.currentTarget, false);
  });

  // 权限下拉（随 agent 联动）
  $('#perm-btn').addEventListener('click', (e) => {
    e.stopPropagation();
    const items = AGENTS[state.agent].permissions.map((p) => ({
      value: p.value,
      label: t(p.label),
      checked: p.value === state.permission,
    }));
    showMenu(e.currentTarget, items, (it) => {
      state.permission = it.value;
      $('#perm-btn').textContent = permLabel();
      savePrefs();
    });
  });

  // 思考等级下拉（等级列表与默认值来自 /api/models）
  $('#effort-btn').addEventListener('click', async (e) => {
    e.stopPropagation();
    if (state.sageOn) {
      showToast(CUR_LANG === 'en' ? 'SAGE chooses reasoning effort automatically' : '智能路由开启时，思考强度由 SAGE 自动选择');
      return;
    }
    const anchor = e.currentTarget;
    let info = { efforts: [], default_effort: null };
    try {
      const all = await getModels();
      info = all[currentAgent()] || info;
    } catch (_) {
      /* 发现失败时提供标准等级 */
    }
    const efforts = info.efforts && info.efforts.length ? info.efforts : ['low', 'medium', 'high'];
    const items = [
      {
        value: null,
        label: '默认',
        hint: info.default_effort ? t(EFFORT_LABELS[info.default_effort] || info.default_effort) : '',
        checked: state.effort === null,
      },
    ];
    for (const ef of efforts) {
      items.push({
        value: ef,
        label: t(EFFORT_LABELS[ef] || ef) + '（' + ef + '）',
        hint: ef === info.default_effort ? t('全局默认') : '',
        checked: ef === state.effort,
      });
    }
    showMenu(anchor, items, (it) => {
      state.effort = it.value;
      syncAgentUI(); // 刷新思考标签与快速按钮联动状态
      savePrefs();
    });
  });

  // 模型下拉（自动从本地 CLI 配置与历史发现；随 agent 联动）
  $('#model-btn').addEventListener('click', async (e) => {
    e.stopPropagation();
    const anchor = e.currentTarget;
    let info = { default: null, models: [] };
    try {
      const all = await getModels();
      info = all[currentAgent()] || info;
    } catch (_) {
      /* 发现失败时仍提供 默认/自定义 */
    }
    // 支持 1M 上下文的模型加徽标（claude 按 [1m] 名字、codex 读目录 context_window）
    const oneM = (m) =>
      m &&
      (((info.windows || {})[m] || 0) >= 1000000 || /\[1m\]/i.test(m))
        ? '1M'
        : '';
    const items = [
      {
        value: null,
        label: t('默认模型'),
        hint: info.default || '',
        tag: oneM(info.default),
        checked: state.model === null,
      },
    ];
    for (const m of info.models) {
      items.push({
        value: m,
        label: m,
        tag: oneM(m),
        hint: m === info.default ? t('全局默认') : '',
        checked: m === state.model,
      });
    }
    if (state.model && !info.models.includes(state.model)) {
      items.splice(1, 0, { value: state.model, label: state.model, tag: oneM(state.model), checked: true });
    }
    items.push({ value: '__custom__', label: '自定义…' });
    showMenu(anchor, items, (it) => {
      if (it.value === '__custom__') {
        const v = prompt(t('输入模型名'));
        if (v && v.trim()) state.model = v.trim();
      } else {
        state.model = it.value;
      }
      $('#model-btn').textContent = modelLabel();
      savePrefs();
    });
  });

  // 项目选择器（Hero）
  $('#project-btn').addEventListener('click', (e) => {
    e.stopPropagation();
    const items = state.projects.map((p) => ({
      value: p.path,
      label: p.name || p.path,
      hint: p.path,
      checked: clientNorm(state.project) === clientNorm(p.path),
    }));
    items.push({ value: '__browse__', label: t('＋ 导入项目…') });
    const anchor = e.currentTarget;
    showMenu(anchor, items, (it) => {
      if (it.value === '__browse__') openProjectPicker(anchor);
      else setProject(it.value);
    });
  });

  // 侧栏
  $('#btn-new-session').addEventListener('click', onNewSession);
  $('#btn-new-session-2').addEventListener('click', onNewSession);

  // 子会话头部：返回主会话 / 回注主会话
  $('#back-primary-btn').addEventListener('click', () => {
    if (!state.backPrimary || !state.session) return;
    const i = state.backPrimary.indexOf(':');
    openSession({
      agent: state.backPrimary.slice(0, i),
      id: state.backPrimary.slice(i + 1),
      project: state.session.project,
      title: '',
    });
  });
  $('#feed-primary-btn').addEventListener('click', feedBackToPrimary);
  $('#btn-add-project').addEventListener('click', (e) => {
    e.stopPropagation();
    openProjectPicker(e.currentTarget);
  });
  $('#head-projects').addEventListener('click', () => toggleGroup('projects'));
  $('#head-convs').addEventListener('click', () => toggleGroup('convs'));

  // 设置：语言切换
  $('#settings-btn').addEventListener('click', (e) => {
    e.stopPropagation();
    const stored = localStorage.getItem('ah-lang');
    showMenu(
      e.currentTarget,
      [
        {
          value: 'auto',
          label: t('跟随浏览器'),
          hint: browserLang() === 'zh' ? '简体中文' : 'English',
          checked: !stored,
        },
        { value: 'zh', label: '简体中文', checked: stored === 'zh' },
        { value: 'en', label: 'English', checked: stored === 'en' },
      ],
      (it) => setLang(it.value)
    );
  });

  // 左侧 agent 过滤图标栏（再点已选中的 = 取消过滤回到全部）
  document.querySelectorAll('.rail-btn[data-filter]').forEach((b) => {
    b.addEventListener('click', () => {
      const f = b.dataset.filter || '';
      setAgentFilter(f === state.agentFilter ? '' : f);
    });
  });
  $('#search-input').addEventListener('input', debounce(onSearch, 250));

  // 多窗口同步：别的窗口改了权限/模型/思考/快速，本窗口立即跟随，
  // 否则旧窗口后续任一次保存会把过期值写回去（快速开关"回答后失效"的根因）
  window.addEventListener('storage', (e) => {
    if (e.key === 'ah-prefs') {
      loadAgentPrefs(currentAgent());
      syncAgentUI();
    }
  });

  // 键盘导航：↑/↓ 平滑滚动对话区；← 子会话退回主会话，主会话退到新建。
  // 输入框有内容时不接管（保留光标移动）；空的主输入框视为页面导航。
  document.addEventListener('keydown', (e) => {
    if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
    const tg = e.target;
    const typing =
      tg && (tg.tagName === 'TEXTAREA' || tg.tagName === 'INPUT' || tg.isContentEditable);
    if (typing && !(tg === promptInput && !promptInput.value)) return;
    if (!state.session) return; // 仅对话视图
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      e.preventDefault();
      chatScrollEl.scrollBy({ top: e.key === 'ArrowDown' ? 140 : -140, behavior: 'smooth' });
    } else if (e.key === 'ArrowLeft') {
      if (state.streaming) return; // 运行中不切走
      e.preventDefault();
      if (state.backPrimary) {
        const i = state.backPrimary.indexOf(':');
        openSession({
          agent: state.backPrimary.slice(0, i),
          id: state.backPrimary.slice(i + 1),
          project: state.session.project,
          title: '',
        });
      } else {
        onNewSession(); // 主会话 → 退到新建会话
      }
    }
  });

  // 侧栏宽度拖拽（分隔条左右滑动，宽度持久化）
  (() => {
    const rez = $('#side-resizer');
    const sb = $('#sidebar');
    if (!rez || !sb) return;
    const saved = parseInt(localStorage.getItem('ah-sidebar-w') || '', 10);
    if (saved >= 220 && saved <= 640) sb.style.width = saved + 'px';
    let dragging = false;
    rez.addEventListener('mousedown', (e) => {
      e.preventDefault();
      dragging = true;
      rez.classList.add('dragging');
      document.body.style.userSelect = 'none';
      document.body.style.cursor = 'col-resize';
    });
    document.addEventListener('mousemove', (e) => {
      if (!dragging) return;
      const w = Math.max(220, Math.min(640, e.clientX));
      sb.style.width = w + 'px';
    });
    document.addEventListener('mouseup', () => {
      if (!dragging) return;
      dragging = false;
      rez.classList.remove('dragging');
      document.body.style.userSelect = '';
      document.body.style.cursor = '';
      localStorage.setItem('ah-sidebar-w', parseInt(sb.style.width, 10) || 300);
    });
  })();

  // 菜单关闭
  document.addEventListener('click', () => closeMenu());
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      closeMenu();
      closeLightbox();
    }
  });
  window.addEventListener('resize', () => closeMenu());
}

/** 页面加载时：若有后台任务在跑，自动打开对应会话并接上输出流 */
async function checkActiveRuns() {
  try {
    const runs = await api.get('/api/runs');
    // 优先主会话运行（协作子运行的 prompt 以【协作 开头），避免开局落到子会话
    const active = runs.filter((x) => x.running && x.session_id);
    const r = active.find((x) => !/^【协作/.test(x.prompt || '')) || active[0];
    if (r) {
      openSession({
        agent: r.agent,
        id: r.session_id,
        project: r.project,
        title: snippet(r.prompt, 40),
      }); // openSession 内部会发现活跃运行并 attachRun
    }
  } catch (_) {
    /* 忽略 */
  }
}

/* ---------- 明亮 / 黑暗主题 ---------- */

function applyTheme(t, persist) {
  document.documentElement.dataset.theme = t === 'light' ? 'light' : 'dark';
  if (persist !== false) localStorage.setItem('ah-theme', t);
  const btn = $('#theme-btn');
  if (btn) btn.textContent = t === 'light' ? '🌙' : '🌓';
}

function init() {
  // 语言：?lang= 参数（调试用）> 记忆值 > 浏览器语言
  const urlLang = new URLSearchParams(location.search).get('lang');
  CUR_LANG = urlLang || localStorage.getItem('ah-lang') || browserLang();
  bindEvents();
  applyLang();
  // 主题：?theme= 参数（调试用，不落存储）> 记忆值 > 暗色
  const urlTheme = new URLSearchParams(location.search).get('theme');
  applyTheme(urlTheme || localStorage.getItem('ah-theme') || 'dark', !urlTheme);
  $('#theme-btn').addEventListener('click', () => {
    const cur = document.documentElement.dataset.theme === 'light' ? 'dark' : 'light';
    applyTheme(cur);
  });
  applyGroupCollapse();
  // 恢复上次的 agent 过滤选择
  document.querySelectorAll('.rail-btn[data-filter]').forEach((b) => {
    b.classList.toggle('active', (b.dataset.filter || '') === state.agentFilter);
  });
  if (state.agentFilter && AGENTS[state.agentFilter]) state.agent = state.agentFilter;
  loadAgentPrefs(state.agent); // 恢复该 agent 上次的权限/模型/思考选择
  syncAgentUI();
  setSendButton(false);
  autoGrow();
  showHero();
  loadStatus();
  getEditors();
  loadProjects();
  loadConvs();
  const restoredAfterReload = restoreReloadState();
  if (!restoredAfterReload) checkActiveRuns();
  checkServerInstance();
  pollRuns();
  setInterval(pollRuns, 5000); // 侧栏运行状态标识轮询
  setInterval(checkServerInstance, 3000); // 部署后旧标签页自动刷新到新实例
  // 相对时间原地刷新：渲染时算好的「刚刚/N 分钟前」不会自己走，
  // 页面久挂不重载列表就会停在旧值——每分钟按 data-ts 重算一遍
  setInterval(() => {
    document.querySelectorAll('.srow-time[data-ts]').forEach((n) => {
      if (n.dataset.ts) n.textContent = relTime(n.dataset.ts);
    });
  }, 60000);
  // 预取模型信息：让「默认」态直接展示实际默认模型与思考强度
  getModels()
    .then((all) => {
      state.modelsInfo = all;
      syncAgentUI();
      // 模型窗口表就位后重渲染用量条——修：刷新后转录先渲染时查不到
      // 每模型窗口，按 200k 误算占比，表到位后又按 1M 算，来回跳变
      if (!stream && state.histUsage && state.session) {
        renderUsageFromHistory(state.histUsage, state.session.agent);
      }
    })
    .catch(() => {});
}

init();
