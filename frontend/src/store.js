// Zustand store — single store, several logical slices.
// Kept mutation-friendly so window.__onEvent can call actions directly from
// outside the React tree.

import { create } from 'zustand';

let msgIdSeq = 1;
const nextId = () => `m${msgIdSeq++}`;

const initialState = {
  // ── session (per-project, hydrated by session_snapshot event)
  session: {
    mode: 'ambiguous',       // 'landing' | 'app' | 'ambiguous'
    brief: '',
    currentProject: null,   // { slug, name } | null
    currentPage: 'home',
    tabs: [],               // [{ slug, name }]
  },

  // Last start_design payload the user submitted, kept so we can re-fire
  // it after they answer a mode-clarify chip. Cleared on success or on
  // project switch.
  lastStartDesignPayload: null,

  // Canvas-side kit picker state. When non-null the Canvas replaces the
  // iframe with the KitPickerStage wizard (one question at a time). On
  // completion the wizard fires assemble_design directly and clears this.
  kitPicker: null,          // { idea, step, picks } | null

  // ── chat messages: [{id, role, label, kind, content, meta}]
  //   role   : 'user' | 'assistant' | 'status' | 'system'
  //   kind   : 'text' | 'kit_picker' | 'pre_page_picker' | 'page_suggestions'
  //          | 'critique' | 'progress' | 'status_ephemeral'
  //   content: string OR object depending on kind
  //   meta   : arbitrary bag (kit picks etc.)
  messages: [],

  // ── status pill + generating flag
  status: {
    state: 'ready',   // 'ready' | 'busy'
    text:  'Ready',
    generating: false,
  },

  // ── tokens
  tokens: {
    turnIn: 0, turnOut: 0,
    sessionIn: 0, sessionOut: 0,
    estimated: false,
  },

  // ── other
  currentHTML: '',            // current design HTML (source of truth)
  assemblyPreview: '',        // interim preview during assembly (NOT persisted)
  projects: [],               // [{slug, name, updated_at, size}]
  meta: { provider: '—', model: '—' },
  critique: null,             // last critique fixes array [{label, prompt}]
  pendingIdea: null,          // idea captured, waiting for kit-picker Send
  currentTarget: null,        // { selector, tag } — element selected in canvas
  crumb: 'new design',
  editReady: false,           // canvas edit script loaded (drives "Live canvas" badge)
  historyResetToken: 0,       // increments on every backend-issued new design; Canvas uses this (not currentHTML) to reset its undo stack
};

export const useStore = create((set, get) => ({
  ...initialState,

  // ── generic setters
  setMeta: (provider, model) => set({ meta: {
    provider: (provider || '—').toLowerCase(),
    model:    (model && model.length) ? model : 'default',
  }}),

  setStatus: (state, text) => set({ status: { ...get().status, state, text }}),
  setGenerating: (v) => set({ status: { ...get().status, generating: !!v }}),

  setTokens: ({ turn_input=0, turn_output=0, session_input=0, session_output=0, estimated=false }) => set({
    tokens: {
      turnIn: turn_input, turnOut: turn_output,
      sessionIn: session_input, sessionOut: session_output,
      estimated,
    },
  }),

  setProjects: (items) => set({ projects: Array.isArray(items) ? items : [] }),

  setPages: (pages, active) => set({
    session: {
      ...get().session,
      tabs: Array.isArray(pages) ? pages : [],
      currentPage: active || 'home',
    },
  }),

  setCurrentProject: (proj) => set({
    session: { ...get().session, currentProject: proj },
    crumb: proj ? proj.name : 'new design',
  }),

  // Session snapshot received from Rust on project open/switch.
  applySessionSnapshot: ({ mode, brief, tokens_in = 0, tokens_out = 0 }) => set(({ session, tokens }) => ({
    session: { ...session, mode: mode || 'ambiguous', brief: brief || '' },
    tokens:  { ...tokens, sessionIn: tokens_in, sessionOut: tokens_out, turnIn: 0, turnOut: 0 },
    lastStartDesignPayload: null,
  })),

  setMode: (mode) => set(({ session }) => ({
    session: { ...session, mode: mode || 'ambiguous' },
  })),

  setLastStartDesign: (payload) => set({ lastStartDesignPayload: payload }),

  // ── Kit picker (canvas-side wizard)
  openKitPicker: (idea) => set({ kitPicker: { idea, step: 0, picks: {} } }),
  updateKitPicker: (patch) => set(({ kitPicker }) => ({
    kitPicker: kitPicker ? { ...kitPicker, ...patch } : null,
  })),
  updateKitPick: (cat, id) => set(({ kitPicker }) => ({
    kitPicker: kitPicker ? { ...kitPicker, picks: { ...kitPicker.picks, [cat]: id } } : null,
  })),
  closeKitPicker: () => set({ kitPicker: null }),

  setCurrentHTML: (html) => set({ currentHTML: html || '' }),

  // Called from ipc.js when a genuinely NEW design lands (design event or
  // project open). Increments a monotonic token that Canvas keys on to
  // reset its undo history — decoupled from currentHTML so user edits
  // don't wipe the stack.
  bumpHistoryReset: () => set(({ historyResetToken }) => ({
    historyResetToken: historyResetToken + 1,
  })),

  setAssemblyPreview: (html) => set({ assemblyPreview: html || '' }),

  setEditReady: (v) => set({ editReady: !!v }),

  setPendingIdea: (idea) => set({ pendingIdea: idea }),

  setCurrentTarget: (t) => set({ currentTarget: t }),

  setCritique: (items) => set({ critique: Array.isArray(items) ? items : null }),

  // ── message actions
  addMessage: (msg) => set(({ messages }) => ({
    messages: [...messages, { id: nextId(), ...msg }],
  })),

  addUser:      (text)  => get().addMessage({ role: 'user',      label: 'You',       kind: 'text', content: text }),
  addAssistant: (text)  => get().addMessage({ role: 'assistant', label: 'Assistant', kind: 'text', content: text }),
  addStatus:    (text)  => {
    // Ephemeral status "bubble" — replace any prior ephemeral status.
    const list = get().messages.filter(m => m.kind !== 'status_ephemeral');
    set({ messages: [...list, { id: nextId(), role: 'status', kind: 'status_ephemeral', content: text }] });
  },
  clearEphemeralStatus: () => set(({ messages }) => ({
    messages: messages.filter(m => m.kind !== 'status_ephemeral'),
  })),

  addKitPicker: (idea) => get().addMessage({
    role: 'assistant', label: 'Assistant · Kit', kind: 'kit_picker',
    content: idea, meta: { picks: {} },
  }),
  removeKitPicker: () => set(({ messages }) => ({
    messages: messages.filter(m => m.kind !== 'kit_picker'),
  })),

  addPrePagePicker: (idea) => {
    // Replace any prior pre_page_picker
    const list = get().messages.filter(m => m.kind !== 'pre_page_picker');
    set({ messages: [...list, { id: nextId(), role: 'assistant', label: 'Assistant', kind: 'pre_page_picker', content: idea }] });
  },
  removePrePagePicker: () => set(({ messages }) => ({
    messages: messages.filter(m => m.kind !== 'pre_page_picker'),
  })),

  addModeClarify: (brief) => {
    const list = get().messages.filter(m => m.kind !== 'mode_clarify');
    set({ messages: [...list, { id: nextId(), role: 'assistant', label: 'Assistant', kind: 'mode_clarify', content: brief }] });
  },
  removeModeClarify: () => set(({ messages }) => ({
    messages: messages.filter(m => m.kind !== 'mode_clarify'),
  })),

  addPageSuggestions: (candidates) => get().addMessage({
    role: 'assistant', label: 'Assistant', kind: 'page_suggestions',
    content: candidates, meta: { consumed: false },
  }),

  addCritique: (items) => get().addMessage({
    role: 'assistant', label: 'Critique', kind: 'critique',
    content: items, meta: { appliedIdx: new Set() },
  }),

  addProgress: () => {
    // Streaming progress card — replace any prior progress card.
    const list = get().messages.filter(m => m.kind !== 'progress');
    set({ messages: [...list, { id: nextId(), role: 'assistant', kind: 'progress', content: { kb: 0, sections: [] } }] });
  },
  updateProgress: (patch) => set(({ messages }) => ({
    messages: messages.map(m => m.kind === 'progress' ? { ...m, content: { ...m.content, ...patch }} : m),
  })),
  removeProgress: () => set(({ messages }) => ({
    messages: messages.filter(m => m.kind !== 'progress'),
  })),

  resetMessages: () => set({ messages: [] }),

  // Reset canvas / picker / target — used when a project closes.
  resetCanvas: () => set({
    currentHTML: '', assemblyPreview: '',
    currentTarget: null, pendingIdea: null, critique: null,
    editReady: false,
    session: { ...get().session, tabs: [], currentPage: 'home' },
  }),
}));

// Convenience: raw getState / setState for out-of-tree callers (IPC dispatcher).
export const storeApi = useStore;
