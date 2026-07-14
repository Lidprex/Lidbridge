'use client'

import { useEffect, useState, useRef } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

function resolveAvatar(url: string): string {
  if (!url) return '/logo.png'
  if (url.startsWith('http://') || url.startsWith('https://')) return url
  return convertFileSrc(url)
}

export interface User {
  id: number
  github_id: string
  email: string
  name: string
  avatar_url: string
}

export interface CleanProgress {
  phase: string
  current: number
  total: number
  current_file: string
  percentage: number
  bytes_copied: number
  deleted_count: number
}

export interface ScanResult {
  total_files: number
  clean_files: number
  skipped_dirs: number
  skipped_files: number
  total_size: number
  clean_size: number
  total_lines?: number
  project_type: string
  skippable: Record<string, number>
  secrets_count: number
  secret_matches: string[]
  secret_suggestions: string[]
}

export interface CleanResult {
  success: boolean
  cleaned_path: string
  copied_files: number
  skipped_files: number
  deleted_items: string[]
  warnings: string[]
  total_size_bytes: number
  scan_result: ScanResult
}

export interface RepoConfig {
  name: string
  description: string
  is_private: boolean
  include_images: boolean
  create_readme: boolean
  license_template: string
  repo_type: string
}

export interface ProgressState {
  percentage: number
  message: string
}

interface DashboardUIProps {
  user: User | null
  loading: boolean
  selectedPath: string
  cleanedPath: string
  scanning: boolean
  cleaning: boolean
  cleaned: boolean
  pushing: boolean
  progress: ProgressState
  toast: { message: string; type: 'success' | 'error' | 'warning' } | null
  repoHistory: Array<{ repo_name: string; repo_url: string; owner_type: string; owner_name: string; created_at: string }>
  scanResult: ScanResult | null
  runLog: string[]
  targetPath: string
  includeImages: boolean
  createReadme: boolean
  showAIModal: boolean
  showAboutModal: boolean
  showHistory: boolean
  isRTL: boolean
  t: (key: string) => string
  setIncludeImages: (include: boolean) => void
  setCreateReadme: (include: boolean) => void
  onLogin: () => void
  onPersonalTokenLogin: (token: string) => Promise<void>
  onLogout: () => void
  onAIAnalysis: () => void
  onAbout: () => void
  onHistory: () => void
  onSelectFolder: () => void
  onSelectTargetFolder: () => void
  onClean: () => void
  onPush: (config: RepoConfig, ownerType: string, ownerName: string) => void
  onCloseToast: () => void
  onCloseAIModal: () => void
  onCloseAboutModal: () => void
  onCloseHistoryModal: () => void
  onOpenSecretReview: () => void
  secretReplacements: Record<string, string>
  onSecretReplacementChange: (secretName: string, value: string) => void
  onApplySecretReplacements: () => void
  pushResultUrl: string
  onResetAfterPush: () => void
  lang: string
  setLang: (lang: string) => void
}

function Header({ user, onLogin, onLogout, onAIAnalysis, onAbout, onHistory, t, lang, setLang }: { user: User | null; onLogin: () => void; onLogout: () => void; onAIAnalysis: () => void; onAbout: () => void; onHistory: () => void; t: (key: string) => string; lang: string; setLang: (l: string) => void }) {
  const [menuOpen, setMenuOpen] = useState(false)
  const [langOpen, setLangOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)
  const langTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const isRTL = lang === 'ar'

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false)
      }
    }
    if (menuOpen) document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [menuOpen])

  const minimize = (e: React.MouseEvent) => { e.stopPropagation(); getCurrentWindow().minimize() }
  const maximize = (e: React.MouseEvent) => { e.stopPropagation(); getCurrentWindow().toggleMaximize() }
  const closeWindow = (e: React.MouseEvent) => { e.stopPropagation(); getCurrentWindow().close() }
  const startDrag = (e: React.MouseEvent) => {
    if (e.button !== 0) return
    if ((e.target as HTMLElement).closest('button, [data-no-drag]')) return
    getCurrentWindow().startDragging()
  }
  const startResize = (dir: 'East' | 'West' | 'South' | 'SouthEast' | 'SouthWest') => (e: React.MouseEvent) => {
    e.stopPropagation()
    getCurrentWindow().startResizeDragging(dir)
  }

  return (
    <>
      <div aria-hidden="true" className="fixed inset-x-2 bottom-0 z-[100] h-1 cursor-s-resize" onMouseDown={startResize('South')} />
      <div aria-hidden="true" className="fixed inset-y-2 left-0 z-[100] w-1 cursor-w-resize" onMouseDown={startResize('West')} />
      <div aria-hidden="true" className="fixed inset-y-2 right-0 z-[100] w-1 cursor-e-resize" onMouseDown={startResize('East')} />
      <div aria-hidden="true" className="fixed bottom-0 right-0 z-[100] h-4 w-4 cursor-se-resize" onMouseDown={startResize('SouthEast')} />
      <div aria-hidden="true" className="fixed bottom-0 left-0 z-[100] h-4 w-4 cursor-sw-resize" onMouseDown={startResize('SouthWest')} />

      <header className="sticky top-0 z-[70] select-none" style={{ backgroundColor: '#1a1a22' }}>
        <div className="flex h-12 items-center justify-between px-2" onMouseDown={startDrag}>
          <div className="flex items-center gap-2" ref={menuRef} data-no-drag>
            <button onClick={() => setMenuOpen(!menuOpen)} className="flex h-8 w-8 items-center justify-center rounded-md text-text-muted transition hover:bg-white/10 hover:text-text-primary" data-no-drag>
              <svg className="h-4 w-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24"><path strokeLinecap="round" d="M4 6h16M4 12h16M4 18h16" /></svg>
            </button>
            {menuOpen && (
              <div className={`absolute top-10 z-[80] w-52 rounded-lg border border-border-subtle bg-bg-secondary shadow-2xl ${isRTL ? 'right-2' : 'left-2'}`}>
                <div className="p-1">
                  {user && (
                    <div className="flex items-center gap-2 px-3 py-2 border-b border-border-subtle mb-1">
                      <img src={resolveAvatar(user.avatar_url)} alt="" className="h-6 w-6 rounded-full" />
                      <div className="min-w-0">
                        <p className="truncate text-xs font-medium text-text-primary">{user.name || user.email}</p>
                        <p className="truncate text-[10px] text-text-muted">{user.email}</p>
                      </div>
                    </div>
                  )}
                  <button onClick={() => { onAbout(); setMenuOpen(false) }} className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-text-secondary hover:bg-bg-tertiary hover:text-text-primary">{t('about')}</button>
                  {user && <>
                    <button onClick={() => { onHistory(); setMenuOpen(false) }} className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-text-secondary hover:bg-bg-tertiary hover:text-text-primary">{t('history')}</button>
                    <button onClick={() => { onAIAnalysis(); setMenuOpen(false) }} className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-text-secondary hover:bg-bg-tertiary hover:text-text-primary">{t('ai_analysis')}</button>
                  </>}
                  <div
                    className="relative"
                    onMouseEnter={() => { if (langTimerRef.current) { clearTimeout(langTimerRef.current); langTimerRef.current = null } setLangOpen(true) }}
                    onMouseLeave={() => { langTimerRef.current = setTimeout(() => setLangOpen(false), 150) }}
                  >
                    <button className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-text-secondary hover:bg-bg-tertiary hover:text-text-primary">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" /></svg>
                      <span className={`flex-1 ${isRTL ? 'text-right' : 'text-left'}`}>{t('language')}</span>
                      <svg className={`w-3 h-3 text-text-muted transition-transform ${langOpen ? (isRTL ? '-rotate-90' : 'rotate-90') : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" /></svg>
                    </button>
                    {langOpen && (
                      <div className={`absolute top-0 w-44 rounded-lg border border-border-subtle bg-bg-secondary shadow-2xl z-[90] p-1 ${isRTL ? 'right-full mr-1' : 'left-full ml-1'}`}>
                        {[
                          { code: 'en', name: 'English', flag: '🇺🇸' },
                          { code: 'ar', name: 'العربية', flag: '🇸🇦' },
                          { code: 'ru', name: 'Русский', flag: '🇷🇺' },
                          { code: 'fr', name: 'Français', flag: '🇫🇷' },
                          { code: 'hi', name: 'हिन्दी', flag: '🇮🇳' },
                          { code: 'zh', name: '中文', flag: '🇨🇳' },
                        ].map(l => (
                          <button key={l.code} onClick={() => { setLang(l.code); setMenuOpen(false); setLangOpen(false) }} className={`flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm ${lang === l.code ? 'bg-bg-tertiary text-text-primary' : 'text-text-secondary hover:bg-bg-tertiary hover:text-text-primary'}`}>
                            <span>{l.flag}</span><span>{l.name}</span>
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
                <div className="border-t border-border-subtle p-1">
                  {user ? (
                    <button onClick={() => { onLogout(); setMenuOpen(false) }} className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-error hover:bg-error/10">{t('logout')}</button>
                  ) : (
                    <button onClick={() => { onLogin(); setMenuOpen(false) }} className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-accent-primary hover:bg-accent-primary/10">{t('connect_github')}</button>
                  )}
                </div>
              </div>
            )}
            <div className="flex items-center gap-1.5">
              <img src="/logo.png" alt="LidBridge" className="h-5 w-5 rounded" />
              <span className="text-xs font-semibold text-text-primary">LidBridge</span>
            </div>
          </div>
          <div className="flex h-full" data-no-drag>
            <button onClick={minimize} className="inline-flex h-full w-12 items-center justify-center text-text-muted hover:bg-white/10 active:bg-white/5" data-no-drag>
              <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 6h8" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round"/></svg>
            </button>
            <button onClick={maximize} className="inline-flex h-full w-12 items-center justify-center text-text-muted hover:bg-white/10 active:bg-white/5" data-no-drag>
              <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2" y="2" width="8" height="8" rx="1" stroke="currentColor" strokeWidth="1.1" fill="none"/></svg>
            </button>
            <button onClick={closeWindow} className="inline-flex h-full w-14 items-center justify-center text-text-muted hover:bg-[#c42b1c] hover:text-white active:bg-[#b42618]" data-no-drag>
              <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2.4 2.4l7.2 7.2M9.6 2.4l-7.2 7.2" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round"/></svg>
            </button>
          </div>
        </div>
        <div className="h-px bg-white/8" />
      </header>
    </>
  )
}



function AuthScreen({ onLogin, onPersonalTokenLogin, t }: { onLogin: () => void; onPersonalTokenLogin: (token: string) => Promise<void>; t: (key: string) => string }) {
  const [showTokenInput, setShowTokenInput] = useState(false)
  const [token, setToken] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [connecting, setConnecting] = useState(false)

  const handleConnect = async () => {
    setConnecting(true)
    try {
      await onLogin()
    } finally {
      setConnecting(false)
    }
  }

  const submitToken = async () => {
    if (!token.trim()) return
    setSubmitting(true)
    try { await onPersonalTokenLogin(token.trim()) } finally { setSubmitting(false) }
  }

  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="card max-w-md w-full text-center">
        <img src="/logo.png" alt="LidBridge logo" className="w-16 h-16 rounded-2xl object-cover mx-auto mb-6 shadow-lg shadow-accent-primary/20" />
        <h2 className="text-2xl font-bold text-text-primary mb-2">{t('welcome')}</h2>
        <p className="text-text-secondary mb-8">{t('subtitle')}</p>
        {!showTokenInput ? <div className="space-y-3">
          <button onClick={handleConnect} disabled={connecting} className="btn btn-primary w-full flex items-center justify-center gap-2 disabled:opacity-60">
            {connecting ? (
              <><svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>Connecting...</>
            ) : (
              <><svg className="h-5 w-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 0C5.37 0 0 5.5 0 12.28c0 5.43 3.44 10.03 8.21 11.66.6.12.82-.27.82-.6v-2.3c-3.34.75-4.04-1.47-4.04-1.47-.55-1.43-1.34-1.8-1.34-1.8-1.09-.77.08-.75.08-.75 1.2.09 1.84 1.27 1.84 1.27 1.07 1.88 2.81 1.34 3.49 1.02.11-.8.42-1.35.76-1.66-2.66-.32-5.47-1.39-5.47-6.19 0-1.37.47-2.49 1.24-3.37-.12-.31-.54-1.6.12-3.33 0 0 1.01-.34 3.3 1.29a11.1 11.1 0 0 1 6.01 0c2.29-1.63 3.3-1.29 3.3-1.29.66 1.73.24 3.02.12 3.33.77.88 1.24 2 1.24 3.37 0 4.81-2.81 5.86-5.48 6.18.43.39.81 1.13.81 2.28v3.38c0 .33.22.72.82.6A12.3 12.3 0 0 0 24 12.28C24 5.5 18.63 0 12 0Z" /></svg>{t('connect_github')}</>
            )}
          </button>
          <div className="relative py-1"><div className="border-t border-border-subtle" /><span className="absolute -top-2 left-1/2 -translate-x-1/2 bg-bg-secondary px-2 text-xs text-text-muted">OR</span></div>
          <button onClick={() => setShowTokenInput(true)} className="btn btn-secondary w-full">Use Personal Token</button>
        </div> : <div className="space-y-4 text-left">
          <div><label className="mb-2 block text-sm text-text-secondary">GitHub Personal Access Token</label><input type="password" value={token} onChange={(event) => setToken(event.target.value)} placeholder="ghp_xxxxxxxxxxxx" className="input w-full" /><p className="mt-2 text-xs text-text-muted">Create one at <a href="https://github.com/settings/tokens/new" target="_blank" rel="noreferrer" className="text-accent-primary hover:underline">github.com/settings/tokens/new</a></p><p className="mt-1 text-xs text-text-muted">Classic token scope: <code className="rounded bg-bg-tertiary px-1">repo</code>. Fine-grained token: repository Contents read/write and Metadata read.</p></div>
          <div className="flex gap-3"><button onClick={() => { setShowTokenInput(false); setToken('') }} className="btn btn-secondary flex-1">Cancel</button><button disabled={submitting || !token.trim()} onClick={() => void submitToken()} className="btn btn-primary flex-1">{submitting ? 'Signing in...' : 'Login'}</button></div>
        </div>}
        <div className="mt-6 p-3 rounded-lg border border-yellow-500/30 bg-yellow-500/5">
          <div className="flex items-start gap-2">
            <svg className="w-4 h-4 text-yellow-500 mt-0.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" /></svg>
            <p className="text-xs text-yellow-500/80 leading-relaxed">{t('security_warning')}</p>
          </div>
        </div>
      </div>
    </div>
  )
}

function StepSelectProject({ selectedPath, scanning, scanResult, onSelectFolder, t }: { selectedPath: string; scanning: boolean; scanResult: ScanResult | null; onSelectFolder: () => void; t: (key: string) => string }) {
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  return (
    <div className="card mb-6">
      <h3 className="text-lg font-semibold text-text-primary mb-4">{t('step1')}</h3>
      <div className="flex gap-3 mb-4">
        <button onClick={onSelectFolder} className="btn btn-secondary flex items-center gap-2">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
          {t('browse_folder')}
        </button>
        <div className="flex-1 bg-bg-tertiary border border-border-subtle rounded-md px-4 py-2 text-text-secondary overflow-hidden text-ellipsis">{selectedPath || t('no_folder')}</div>
      </div>
      {scanning && (
        <div className="flex items-center gap-3 p-4 rounded-lg border border-accent-primary/30 bg-accent-primary/10">
          <svg className="w-5 h-5 animate-spin text-accent-primary" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
          <span className="text-sm text-accent-primary">Scanning project files…</span>
        </div>
      )}
      {!scanning && scanResult && selectedPath && (
        <div className="p-4 rounded-lg border border-border-subtle bg-bg-tertiary/50">
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-3">
            <div>
              <p className="text-lg font-bold text-accent-primary">{scanResult.project_type}</p>
              <p className="text-xs text-text-muted">Project Type</p>
            </div>
            <div>
              <p className="text-lg font-bold text-text-primary">{scanResult.total_files}</p>
              <p className="text-xs text-text-muted">Total Files</p>
            </div>
            <div>
              <p className="text-lg font-bold text-text-primary">{formatBytes(scanResult.total_size)}</p>
              <p className="text-xs text-text-muted">Total Size</p>
            </div>
            <div>
              <p className="text-lg font-bold text-accent-secondary">{formatBytes(scanResult.clean_size)}</p>
              <p className="text-xs text-text-muted">Clean Size (upload)</p>
            </div>
          </div>
          <div className="flex flex-wrap gap-3 text-xs text-text-muted">
            <span>{scanResult.clean_files} source files</span>
            <span>•</span>
            <span>{scanResult.skipped_dirs} skipped folders ({scanResult.skipped_files} files)</span>
            {scanResult.total_lines ? <><span>•</span><span>{scanResult.total_lines.toLocaleString()} lines scanned</span></> : null}
            {scanResult.secrets_count > 0 ? <><span>•</span><span className="text-warning">{scanResult.secrets_count} secrets</span></> : null}
          </div>
          {Object.keys(scanResult.skippable).length > 0 && (
            <div className="mt-3 flex flex-wrap gap-1.5">
              {Object.entries(scanResult.skippable).sort((a, b) => b[1] - a[1]).slice(0, 8).map(([name, size]) => (
                <span key={name} className="px-2 py-0.5 rounded-full bg-bg-secondary text-[11px] text-text-muted border border-border-subtle">{name} ({formatBytes(size)})</span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function SecretReviewModal({ isOpen, secretMatches, secretSuggestions, replacements, onClose, onChange, onApply, t }: { isOpen: boolean; secretMatches: string[]; secretSuggestions: string[]; replacements: Record<string, string>; onClose: () => void; onChange: (secretName: string, value: string) => void; onApply: () => void; t: (key: string) => string }) {
  const [editingMatch, setEditingMatch] = useState<string | null>(null)
  const [editorValue, setEditorValue] = useState('')

  if (!isOpen) return null
  return (
    <>
      <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-[120]" onClick={onClose}>
        <div className="bg-bg-secondary border border-border-subtle rounded-2xl p-6 w-full max-w-2xl" onClick={(e) => e.stopPropagation()}>
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-xl font-semibold text-text-primary">Secret review</h3>
              <p className="text-sm text-text-secondary">Replace sensitive values before creating the cleaned output. Changes only affect the upload copy.</p>
            </div>
            <button onClick={onClose} className="text-text-muted hover:text-text-primary">Close</button>
          </div>
          <div className="space-y-3 max-h-[60vh] overflow-y-auto">
            {secretMatches.length === 0 ? (
              <div className="p-4 rounded-lg bg-bg-tertiary text-text-secondary">No secret-like values detected.</div>
            ) : secretMatches.map((match, index) => (
              <div key={match} className="rounded-lg border border-border-subtle p-4 bg-bg-tertiary">
                <div className="flex items-center justify-between gap-3">
                  <div className="font-medium text-text-primary">{match}</div>
                  <button onClick={() => { setEditingMatch(match); setEditorValue(replacements[match] ?? secretSuggestions[index] ?? '') }} className="text-xs font-medium text-accent-primary">Manual edit</button>
                </div>
                <input value={replacements[match] ?? ''} onChange={(e) => onChange(match, e.target.value)} className="input w-full mt-3" placeholder={`Replace ${match}`} />
                <div className="mt-3 flex flex-wrap gap-2">
                  <button onClick={() => onChange(match, secretSuggestions[index] ?? 'your_placeholder')} className="rounded-full border border-accent-primary/30 bg-accent-primary/10 px-3 py-1 text-xs text-accent-primary">Use suggested placeholder</button>
                  <button onClick={() => { setEditingMatch(match); setEditorValue(replacements[match] ?? secretSuggestions[index] ?? '') }} className="rounded-full border border-border-subtle px-3 py-1 text-xs text-text-secondary">Manual edit</button>
                </div>
                <p className="text-xs text-text-muted mt-2">The cleaned output will use this value instead of the detected secret.</p>
              </div>
            ))}
          </div>
          <div className="flex gap-3 mt-6">
            <button onClick={onClose} className="btn btn-secondary flex-1">{t('cancel')}</button>
            <button onClick={onApply} className="btn btn-primary flex-1">Apply replacements</button>
          </div>
        </div>
      </div>
      {editingMatch && <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-[130]" onClick={() => setEditingMatch(null)}>
        <div className="w-full max-w-xl rounded-2xl border border-border-subtle bg-bg-secondary p-6" onClick={(e) => e.stopPropagation()}>
          <div className="mb-4">
            <h4 className="text-lg font-semibold text-text-primary">Edit prepared copy</h4>
            <p className="mt-1 text-sm text-text-secondary">Changes made here only affect the version being prepared for upload. Your original project files are not modified.</p>
          </div>
          <textarea value={editorValue} onChange={(e) => setEditorValue(e.target.value)} className="input min-h-36 w-full" />
          <div className="mt-4 flex justify-end gap-3">
            <button onClick={() => setEditingMatch(null)} className="btn btn-secondary">Close</button>
            <button onClick={() => { onChange(editingMatch, editorValue); setEditingMatch(null) }} className="btn btn-primary">Save for upload copy</button>
          </div>
        </div>
      </div>}
    </>
  )
}

function StepCleanProject({ selectedPath, targetPath, scanning, onSelectTargetFolder, onClean, cleaning, cleaned, scanResult, onOpenSecretReview, secretReplacements, onSecretReplacementChange, onApplySecretReplacements, t }: { selectedPath: string; targetPath: string; scanning: boolean; onSelectTargetFolder: () => void; onClean: () => void; cleaning: boolean; cleaned: boolean; scanResult: ScanResult | null; onOpenSecretReview: () => void; secretReplacements: Record<string, string>; onSecretReplacementChange: (secretName: string, value: string) => void; onApplySecretReplacements: () => void; t: (key: string) => string }) {
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const [showSecretModal, setShowSecretModal] = useState(false)

  return (
    <>
      <div className="card mb-6">
        <h3 className="text-lg font-semibold text-text-primary mb-4">{t('step2')}</h3>
        {scanResult && (
          <div className="mb-4">
            {Object.keys(scanResult.skippable).length > 0 && (
              <div className="mb-4 p-4 rounded-lg border border-border-subtle bg-bg-tertiary/50">
                <p className="text-text-muted text-xs mb-2">{t('files_to_remove')}</p>
                <div className="flex flex-wrap gap-2">
                  {Object.entries(scanResult.skippable).sort((a, b) => b[1] - a[1]).slice(0, 8).map(([name, size]) => (
                    <span key={name} className="px-3 py-1 bg-bg-secondary rounded-full text-xs text-text-muted border border-border-subtle">{name} ({formatBytes(size)})</span>
                  ))}
                </div>
              </div>
            )}
            {(scanResult.secrets_count || 0) > 0 && (
              <div className="rounded-lg border border-warning/40 bg-warning/10 p-4">
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <p className="text-sm font-medium text-warning">Potential secrets detected</p>
                    <p className="text-xs text-text-secondary mt-1">We found {scanResult.secrets_count} potential secret-like values. Review them before publishing.</p>
                  </div>
                  <button onClick={() => setShowSecretModal(true)} className="btn btn-secondary">Review secrets</button>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {scanResult.secret_matches.slice(0, 6).map((match) => (
                    <span key={match} className="px-3 py-1 bg-bg-secondary rounded-full text-xs text-text-muted">{match}</span>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
        <div className="mb-4">
          <div className="flex gap-3">
            <div className="flex-1 p-4 rounded-lg border border-accent-primary bg-accent-primary/10">
              <p className="font-medium text-sm text-text-primary">Smart Clean</p>
              <p className="text-xs text-text-muted">Remove junk and keep your project structure.</p>
            </div>
          </div>
        </div>
        <div className="mb-4">
          <div className="flex gap-3">
            <button onClick={onSelectTargetFolder} className="btn btn-secondary flex items-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
              {t('browse_target_folder')}
            </button>
            <div className="flex-1 bg-bg-tertiary border border-border-subtle rounded-md px-4 py-2 text-text-secondary overflow-hidden text-ellipsis">{targetPath || t('no_target_folder')}</div>
          </div>
        </div>
        {scanning && <div className="mb-4 rounded-lg border border-accent-primary/30 bg-accent-primary/10 p-3 text-sm text-accent-primary">Scanning files… The clean button will activate once the scan completes.</div>}
        <button onClick={onClean} disabled={!selectedPath || cleaning || cleaned || scanning} className="btn btn-primary flex items-center gap-2 disabled:opacity-60">
          {cleaning ? (<><svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>{t('cleaning')}</>) : cleaned ? (<><svg className="w-5 h-5 text-success" fill="none" stroke="currentColor" strokeWidth="2.5" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7"/></svg>{t('cleaning_complete')}</>) : (<><svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>{t('clean_project')}</>) }
        </button>
      </div>
      <SecretReviewModal isOpen={showSecretModal} secretMatches={scanResult?.secret_matches ?? []} secretSuggestions={scanResult?.secret_suggestions ?? []} replacements={secretReplacements} onClose={() => setShowSecretModal(false)} onChange={onSecretReplacementChange} onApply={() => { onApplySecretReplacements(); setShowSecretModal(false) }} t={t} />
    </>
  )
}

function StepPushToGitHub({ cleanedPath, onPush, pushing, t }: { cleanedPath: string; onPush: (config: RepoConfig, ownerType: string, ownerName: string) => void; pushing: boolean; t: (key: string) => string }) {
  const [showModal, setShowModal] = useState(false)
  const [repoName, setRepoName] = useState('')
  const [description, setDescription] = useState('')
  const [isPrivate, setIsPrivate] = useState(true)
  const [includeImages, setIncludeImages] = useState(true)
  const [createReadme, setCreateReadme] = useState(true)
  const [licenseTemplate, setLicenseTemplate] = useState('mit')
  const [repoType, setRepoType] = useState('standard')
  const [ownerType, setOwnerType] = useState<'user' | 'org'>('user')
  const [organizations, setOrganizations] = useState<Array<{login: string, id: number}>>([])
  const [selectedOrg, setSelectedOrg] = useState('')

  useEffect(() => {
    const fetchOrgs = async () => {
      try {
        const orgs = await invoke<Array<{login: string, id: number}>>('get_user_organizations')
        setOrganizations(orgs)
      } catch (err) {
        console.error('Failed to fetch organizations:', err)
      }
    }
    if (showModal) {
      fetchOrgs()
    }
  }, [showModal])

  const handleOpenModal = () => {
    if (cleanedPath) {
      setRepoName(cleanedPath.split(/[/\\]/).pop()?.replace('_LidBridge', '') || '')
      setOwnerType('user')
      setSelectedOrg('')
      setShowModal(true)
    }
  }

  const handlePush = () => {
    const ownerName = ownerType === 'user' ? '' : selectedOrg
    onPush({ name: repoName, description, is_private: isPrivate, include_images: includeImages, create_readme: createReadme, license_template: licenseTemplate, repo_type: repoType }, ownerType, ownerName)
    setShowModal(false)
  }

  return (
    <div className="card mb-6">
      <h3 className="text-lg font-semibold text-text-primary mb-4">{t('step3')}</h3>
      <button onClick={handleOpenModal} disabled={!cleanedPath || pushing} className="btn btn-primary flex items-center gap-2">
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" /></svg>
        {t('push_github')}
      </button>
      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-bg-secondary border border-border-subtle rounded-lg p-6 w-full max-w-md">
            <h3 className="text-xl font-semibold text-text-primary mb-6">{t('create_repo')}</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-text-secondary text-sm mb-2">Push to</label>
                <div className="flex gap-4">
                  <button onClick={() => setOwnerType('user')} className={`flex-1 py-2 rounded-md border ${ownerType === 'user' ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'}`}>My Account</button>
                  <button onClick={() => setOwnerType('org')} disabled={organizations.length === 0} className={`flex-1 py-2 rounded-md border ${ownerType === 'org' ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'} ${organizations.length === 0 ? 'opacity-50 cursor-not-allowed' : ''}`}>Organization</button>
                </div>
              </div>
              {ownerType === 'org' && organizations.length > 0 && (
                <div>
                  <label className="block text-text-secondary text-sm mb-2">Select Organization</label>
                  <select value={selectedOrg} onChange={(e) => setSelectedOrg(e.target.value)} className="input w-full">
                    <option value="">Select organization</option>
                    {organizations.map((org) => (<option key={org.id} value={org.login}>{org.login}</option>))}
                  </select>
                </div>
              )}
              <div><label className="block text-text-secondary text-sm mb-2">{t('repo_name')}</label><input type="text" value={repoName} onChange={(e) => setRepoName(e.target.value)} className="input w-full" placeholder="my-project" /></div>
              <div><label className="block text-text-secondary text-sm mb-2">{t('description')}</label><textarea value={description} onChange={(e) => setDescription(e.target.value)} className="input w-full h-24 resize-none" placeholder="A brief description..." /></div>
              <div><label className="block text-text-secondary text-sm mb-2">License</label><select value={licenseTemplate} onChange={(e) => setLicenseTemplate(e.target.value)} className="input w-full"><option value="mit">MIT</option><option value="apache-2.0">Apache-2.0</option><option value="gpl-3.0">GPL-3.0</option><option value="none">None</option></select></div>
              <div><label className="block text-text-secondary text-sm mb-2">Repository Type</label><select value={repoType} onChange={(e) => setRepoType(e.target.value)} className="input w-full"><option value="standard">Standard</option><option value="template">Template</option></select></div>
              <div><label className="block text-text-secondary text-sm mb-2">{t('visibility')}</label><div className="flex gap-4"><button onClick={() => setIsPrivate(false)} className={`flex-1 py-2 rounded-md border ${!isPrivate ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'}`}>{t('public')}</button><button onClick={() => setIsPrivate(true)} className={`flex-1 py-2 rounded-md border ${isPrivate ? 'bg-accent-primary text-black border-accent-primary' : 'border-border-subtle'}`}>{t('private')}</button></div></div>
              <div className="space-y-2"><label className="flex items-center gap-3 cursor-pointer"><input type="checkbox" checked={includeImages} onChange={(e) => setIncludeImages(e.target.checked)} className="checkbox" /><span className="text-text-secondary">{t('include_images')}</span></label><label className="flex items-center gap-3 cursor-pointer"><input type="checkbox" checked={createReadme} onChange={(e) => setCreateReadme(e.target.checked)} className="checkbox" /><span className="text-text-secondary">{t('create_readme')}</span></label></div>
            </div>
            <div className="flex gap-3 mt-6"><button onClick={() => setShowModal(false)} className="btn btn-secondary flex-1">{t('cancel')}</button><button onClick={handlePush} disabled={!repoName || (ownerType === 'org' && !selectedOrg)} className="btn btn-primary flex-1">{t('create_push')}</button></div>
          </div>
        </div>
      )}
    </div>
  )
}

function ProgressBar({ progress, runLog, t }: { progress: ProgressState; runLog: string[]; t: (key: string) => string }) {
  const [showLog, setShowLog] = useState(false)
  return (
    <>
      <div className="card mt-6">
        <div className="flex items-center justify-between text-text-secondary text-sm mb-2"><span>{progress.message}</span><span>{progress.percentage}%</span></div>
        <div className="progress-bar"><div className="progress-fill" style={{ width: `${progress.percentage}%` }}></div></div>
        <div className="mt-3 flex items-center justify-between rounded-lg border border-border-subtle bg-bg-tertiary/70 px-3 py-2">
          <p className="text-xs text-text-secondary">Current operation: <span className="font-medium text-text-primary">{progress.message || 'Preparing run'}</span></p>
          <button onClick={() => setShowLog(true)} className="text-xs font-medium text-accent-primary">Open full log</button>
        </div>
        <div className="mt-3 rounded-lg border border-border-subtle bg-bg-tertiary/70 p-3">
          <p className="text-[11px] uppercase tracking-[0.2em] text-text-muted">Latest activity</p>
          <div className="mt-2 space-y-1 text-sm text-text-secondary">
            {runLog.slice(-4).reverse().map((item, index) => <p key={`${item}-${index}`} className="truncate">• {item}</p>)}
          </div>
        </div>
      </div>
      {showLog && <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50" onClick={() => setShowLog(false)}><div className="w-full max-w-xl rounded-2xl border border-border-subtle bg-bg-secondary p-6 shadow-2xl" onClick={(e) => e.stopPropagation()}><div className="mb-4 flex items-center justify-between"><h3 className="text-lg font-semibold text-text-primary">Run log</h3><button onClick={() => setShowLog(false)} className="text-sm font-medium text-text-muted hover:text-text-primary transition">Close</button></div><div className="max-h-[60vh] overflow-y-auto rounded-lg bg-bg-tertiary p-4 text-sm text-text-secondary"><div className="space-y-1">{runLog.length === 0 ? <p>No activity recorded yet.</p> : runLog.map((item, index) => <p key={`${item}-${index}`}>• {item}</p>)}</div></div></div></div>}
    </>
  )
}

function Toast({ message, type, onClose }: { message: string; type: 'success' | 'error' | 'warning'; onClose: () => void }) {
  useEffect(() => {
    const timer = setTimeout(onClose, 8000)
    return () => clearTimeout(timer)
  }, [onClose])
  const colors = { success: 'border-success', error: 'border-error', warning: 'border-warning' }
  return (<div className={`fixed bottom-4 right-4 bg-bg-tertiary border-l-4 ${colors[type]} rounded-md p-4 shadow-lg z-50`}><p className="text-text-primary">{message}</p></div>)
}

function AIAnalysisModal({ isOpen, onClose, t }: { isOpen: boolean; onClose: () => void; t: (key: string) => string }) {
  if (!isOpen) return null
  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-bg-secondary border border-border-subtle rounded-2xl p-8 w-full max-w-lg shadow-2xl" onClick={(e) => e.stopPropagation()}>
        <div className="text-center mb-8">
          <div className="w-16 h-16 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-2xl flex items-center justify-center mx-auto mb-4 shadow-lg shadow-accent-primary/20">
            <svg className="w-8 h-8 text-white" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.5 0 12.28c0 5.43 3.44 10.03 8.21 11.66.6.12.82-.27.82-.6v-2.3c-3.34.75-4.04-1.47-4.04-1.47-.55-1.43-1.34-1.8-1.34-1.8-1.09-.77.08-.75.08-.75 1.2.09 1.84 1.27 1.84 1.27 1.07 1.88 2.81 1.34 3.49 1.02.11-.8.42-1.35.76-1.66-2.66-.32-5.47-1.39-5.47-6.19 0-1.37.47-2.49 1.24-3.37-.12-.31-.54-1.6.12-3.33 0 0 1.01-.34 3.3 1.29a11.1 11.1 0 0 1 6.01 0c2.29-1.63 3.3-1.29 3.3-1.29.66 1.73.24 3.02.12 3.33.77.88 1.24 2 1.24 3.37 0 4.81-2.81 5.86-5.48 6.18.43.39.81 1.13.81 2.28v3.38c0 .33.22.72.82.6A12.3 12.3 0 0 0 24 12.28C24 5.5 18.63 0 12 0Z"/></svg>
          </div>
          <h2 className="text-xl font-bold text-text-primary">{t('ai_analysis_title')}</h2>
          <p className="text-accent-primary text-sm mt-1">{t('ai_analysis_title_sub')}</p>
        </div>

        <div className="space-y-3 mb-6">
          <div className="rounded-xl border border-border-subtle bg-bg-primary/50 p-5">
            <div className="flex items-start gap-4">
              <div className="w-10 h-10 bg-accent-primary/15 rounded-xl flex items-center justify-center flex-shrink-0">
                <svg className="w-5 h-5 text-accent-primary" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
              </div>
              <div>
                <h4 className="text-text-primary font-semibold">Smart README Generation</h4>
                <p className="text-text-secondary text-sm mt-1">AI generates detailed documentation and file structure analysis</p>
              </div>
            </div>
          </div>

          <div className="rounded-xl border border-border-subtle bg-bg-primary/50 p-5">
            <div className="flex items-start gap-4">
              <div className="w-10 h-10 bg-warning/15 rounded-xl flex items-center justify-center flex-shrink-0">
                <svg className="w-5 h-5 text-warning" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
              </div>
              <div>
                <h4 className="text-text-primary font-semibold">Security Scan</h4>
                <p className="text-text-secondary text-sm mt-1">Detects exposed API keys or secrets before pushing to GitHub</p>
              </div>
            </div>
          </div>
        </div>

        <div className="rounded-xl border border-accent-primary/30 bg-accent-primary/5 p-5 mb-6">
          <div className="flex items-start gap-3">
            <span className="text-lg flex-shrink-0"></span>
            <div>
              <p className="text-text-primary text-sm font-medium">{t('ai_analysis_pro')}</p>
              <p className="text-text-secondary text-sm mt-2 leading-relaxed">As an independent developer, I cannot provide this feature for free. GitHub Copilot API costs are high, and this helps me maintain the app for everyone.</p>
            </div>
          </div>
        </div>

        <button onClick={onClose} className="btn btn-secondary w-full">Close</button>
      </div>
    </div>
  )
}

function HistoryModal({ isOpen, repos, onClose, t }: { isOpen: boolean; repos: Array<{ repo_name: string; repo_url: string; owner_type: string; owner_name: string; created_at: string }>; onClose: () => void; t: (key: string) => string }) {
  if (!isOpen) return null
  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-bg-secondary border border-border-subtle rounded-2xl p-6 w-full max-w-3xl" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-6"><div><h2 className="text-2xl font-bold text-text-primary">{t('history_title')}</h2><p className="text-text-secondary text-sm">{t('your_repositories')}</p></div><button onClick={onClose} className="text-text-muted hover:text-text-primary">Close</button></div>
        <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-2">{repos.length === 0 ? <div className="p-6 bg-bg-primary rounded-xl text-text-secondary text-sm">No repository history found.</div> : repos.map((repo, idx) => (<a key={idx} href={repo.repo_url} target="_blank" rel="noreferrer" className="block p-4 border border-border-subtle rounded-xl hover:border-accent-primary transition"><div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2"><div><p className="text-text-primary font-semibold">{repo.repo_name}</p><p className="text-text-secondary text-sm">{t('owner')}: {repo.owner_name}</p></div><div className="text-text-muted text-sm">{repo.created_at}</div></div></a>))}</div>
      </div>
    </div>
  )
}

function AboutModal({ isOpen, onClose, t }: { isOpen: boolean; onClose: () => void; t: (key: string) => string }) {
  if (!isOpen) return null
  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-gradient-to-br from-bg-secondary to-bg-tertiary border border-accent-secondary/30 rounded-2xl p-8 w-full max-w-md" onClick={(e) => e.stopPropagation()}>
        <div className="text-center mb-6"><img src="/logo.png" alt="LidBridge logo" className="w-24 h-24 mx-auto rounded-2xl mb-4" /><h2 className="text-2xl font-bold text-text-primary">{t('about_title')}</h2><p className="text-text-secondary text-sm mt-1">v2.0.0</p></div>
        <div className="text-center mb-6"><p className="text-text-muted text-sm">{t('created_by')}</p><p className="text-xl font-bold bg-gradient-to-r from-accent-primary to-accent-secondary bg-clip-text text-transparent">Lidprex Labs</p></div>
        <div className="space-y-2"><a href="https://lidprex.onrender.com/" target="_blank" rel="noreferrer" className="block rounded-lg border border-border-subtle p-3 text-text-secondary hover:border-accent-secondary">{t('parent_company')}</a><a href="https://lidprex-labs.onrender.com/" target="_blank" rel="noreferrer" className="block rounded-lg border border-border-subtle p-3 text-text-secondary hover:border-accent-secondary">{t('the_lab')}</a><a href="https://github.com/bxat01" target="_blank" rel="noreferrer" className="block rounded-lg border border-border-subtle p-3 text-text-secondary hover:border-accent-secondary">{t('lead_developer')}</a><a href="https://github.com/lidprex" target="_blank" rel="noreferrer" className="block rounded-lg border border-border-subtle p-3 text-text-secondary hover:border-accent-secondary">{t('organization')}</a></div>
        <button onClick={onClose} className="btn btn-secondary w-full mt-6">Close</button>
      </div>
    </div>
  )
}

export function DashboardUI(props: DashboardUIProps) {
  const { user, loading, selectedPath, cleanedPath, scanning, cleaning, cleaned, pushing, progress, toast, repoHistory, scanResult, runLog, targetPath, includeImages, createReadme, showAIModal, showAboutModal, showHistory, isRTL, t, setIncludeImages, setCreateReadme, onLogin, onPersonalTokenLogin, onLogout, onAIAnalysis, onAbout, onHistory, onSelectFolder, onSelectTargetFolder, onClean, onPush, onCloseToast, onCloseAIModal, onCloseAboutModal, onCloseHistoryModal, onOpenSecretReview, secretReplacements, onSecretReplacementChange, onApplySecretReplacements, pushResultUrl, onResetAfterPush, lang, setLang } = props

  if (loading) return (<div className="min-h-screen bg-bg-primary flex items-center justify-center"><div className="animate-spin w-8 h-8 border-2 border-accent-primary border-t-transparent rounded-full"></div></div>)

  return (
    <div className={`min-h-screen bg-bg-primary flex flex-col ${isRTL ? 'rtl' : 'ltr'}`} dir={isRTL ? 'rtl' : 'ltr'}>
      <Header user={user} onLogin={onLogin} onLogout={onLogout} onAIAnalysis={onAIAnalysis} onAbout={onAbout} onHistory={onHistory} t={t} lang={lang} setLang={setLang} />
      {!user ? <AuthScreen onLogin={onLogin} onPersonalTokenLogin={onPersonalTokenLogin} t={t} /> : (
        <main className="flex-1 overflow-y-auto p-8 max-w-3xl mx-auto w-full" style={{ scrollPaddingTop: '48px' }}>
          {pushResultUrl ? (
            <div className="flex items-center justify-center min-h-[60vh]">
              <div className="card text-center space-y-6 py-8 w-full max-w-md">
                <div className="w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto">
                  <svg className="w-8 h-8 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg>
                </div>
                <h3 className="text-xl font-bold text-text-primary">Push Complete</h3>
                <p className="text-text-secondary">All files have been pushed to GitHub successfully.</p>
                <div className="flex gap-4 justify-center pt-2">
                  <a href={pushResultUrl} target="_blank" rel="noreferrer" className="btn btn-primary flex items-center gap-2">
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
                    Open Repository
                  </a>
                  <button onClick={onResetAfterPush} className="btn btn-secondary flex items-center gap-2">
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
                    Start New Process
                  </button>
                </div>
              </div>
            </div>
          ) : (<>
          <StepSelectProject selectedPath={selectedPath} scanning={scanning} scanResult={scanResult} onSelectFolder={onSelectFolder} t={t} />
          <StepCleanProject selectedPath={selectedPath} targetPath={targetPath} scanning={scanning} onSelectTargetFolder={onSelectTargetFolder} onClean={onClean} cleaning={cleaning} cleaned={cleaned} scanResult={scanResult} onOpenSecretReview={onOpenSecretReview} secretReplacements={secretReplacements} onSecretReplacementChange={onSecretReplacementChange} onApplySecretReplacements={onApplySecretReplacements} t={t} />
          <StepPushToGitHub cleanedPath={cleanedPath} onPush={onPush} pushing={pushing} t={t} />
          </>
          )}
          {(scanning || cleaning || pushing) && <ProgressBar progress={progress} runLog={props.runLog} t={t} />}
        </main>
      )}
      {toast && <Toast message={toast.message} type={toast.type} onClose={onCloseToast} />}
      <AIAnalysisModal isOpen={showAIModal} onClose={onCloseAIModal} t={t} />
      <HistoryModal isOpen={showHistory} repos={repoHistory} onClose={onCloseHistoryModal} t={t} />
      <AboutModal isOpen={showAboutModal} onClose={onCloseAboutModal} t={t} />
    </div>
  )
}
