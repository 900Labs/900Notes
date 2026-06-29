import type { Page, PageTreeNode, PageTreeNodeMeta, PageMetadata, Tag, SearchResult, Backlink, PageProperty, Template, SavedSearch, SmartFolder, PageRevision, Favorite, TagGroup, RelatedPage, Attachment, AudioNote, SyncStatus, SyncDeviceInfo, Workspace } from '../lib/types'
import * as api from '../lib/api'

export class PageStore {
  currentPage = $state<Page | null>(null)
  pageTree = $state<PageTreeNodeMeta[]>([])
  recentPages = $state<PageMetadata[]>([])
  trashPages = $state<Page[]>([])
  isLoading = $state(false)

  async loadPageTree() {
    this.isLoading = true
    try {
      this.pageTree = await api.getPageTreeMetadata()
    } finally {
      this.isLoading = false
    }
  }

  async loadPage(id: string) {
    this.isLoading = true
    try {
      this.currentPage = await api.getPage(id)
    } finally {
      this.isLoading = false
    }
  }

  async createPage(title: string, parentId: string | null): Promise<Page> {
    const page = await api.createPage({ parentId, title, content: undefined })
    await this.loadPageTree()
    return page
  }

  async updatePage(id: string, updates: Partial<{ title: string; content: string; icon: string | null; coverColor: string | null; pinned: boolean }>) {
    const updated = await api.updatePage({ id, ...updates })
    this.currentPage = updated
    const isContentOnly = Object.keys(updates).length === 1 && 'content' in updates
    if (!isContentOnly) {
      await this.loadPageTree()
    }
    return updated
  }

  async deletePage(id: string) {
    await api.deletePage(id)
    if (this.currentPage?.id === id) {
      this.currentPage = null
    }
    await this.loadPageTree()
  }

  async restorePage(id: string) {
    await api.restorePage(id)
    await this.loadPageTree()
    await this.loadTrash()
  }

  async duplicatePage(id: string) {
    const page = await api.duplicatePage(id)
    await this.loadPageTree()
    return page
  }

  async movePage(id: string, parentId: string | null, sortOrder: number) {
    await api.movePage({ id, parentId, sortOrder })
    await this.loadPageTree()
  }

  async loadRecentPages() {
    this.recentPages = await api.getRecentPagesMetadata(10)
  }

  async loadTrash() {
    this.trashPages = await api.getTrash()
  }

  async emptyTrash() {
    await api.emptyTrash()
    await this.loadTrash()
  }

  async search(query: string): Promise<SearchResult[]> {
    if (!query.trim()) return []
    return api.searchPages(query)
  }

  async getAllPages(): Promise<Page[]> {
    return api.getAllPages()
  }
}

export class TagStore {
  tags = $state<Tag[]>([])
  pageTags = $state<Tag[]>([])

  async loadTags() {
    this.tags = await api.getAllTags()
  }

  async loadPageTags(pageId: string) {
    this.pageTags = await api.getPageTags(pageId)
  }

  async createTag(name: string, color: string | null = null): Promise<Tag> {
    const tag = await api.createTag({ name, color })
    await this.loadTags()
    return tag
  }

  async updateTag(id: string, updates: { name?: string; color?: string | null }) {
    const tag = await api.updateTag({ id, ...updates })
    await this.loadTags()
    return tag
  }

  async deleteTag(id: string) {
    await api.deleteTag(id)
    await this.loadTags()
  }

  async setPageTags(pageId: string, tagIds: string[]) {
    await api.setPageTags(pageId, tagIds)
    await this.loadPageTags(pageId)
  }
}

export class BacklinkStore {
  backlinks = $state<Backlink[]>([])

  async loadBacklinks(pageId: string) {
    this.backlinks = await api.getBacklinks(pageId)
  }
}

export class PropertyStore {
  properties = $state<PageProperty[]>([])

  async loadProperties(pageId: string) {
    this.properties = await api.getPageProperties(pageId)
  }

  async setProperty(pageId: string, key: string, value: string) {
    const prop = await api.setPageProperty({ pageId, key, value })
    await this.loadProperties(pageId)
    return prop
  }

  async deleteProperty(id: string, pageId: string) {
    await api.deletePageProperty(id)
    await this.loadProperties(pageId)
  }

  clear() {
    this.properties = []
  }
}

export class TemplateStore {
  templates = $state<Template[]>([])

  async loadTemplates() {
    this.templates = await api.getAllTemplates()
  }

  async createTemplate(name: string, icon: string, content: string, category: string): Promise<Template> {
    const template = await api.createTemplate({ name, icon, content, category })
    await this.loadTemplates()
    return template
  }

  async updateTemplate(id: string, updates: { name?: string; icon?: string; content?: string; category?: string }) {
    const template = await api.updateTemplate({ id, ...updates })
    await this.loadTemplates()
    return template
  }

  async deleteTemplate(id: string) {
    await api.deleteTemplate(id)
    await this.loadTemplates()
  }

  async createPageFromTemplate(templateId: string, title: string, parentId: string | null): Promise<Page> {
    return api.createPageFromTemplate(templateId, title, parentId)
  }

  async getOrCreateDailyNote(date: string): Promise<Page> {
    return api.getOrCreateDailyNote(date)
  }
}

export class SearchStore {
  savedSearches = $state<SavedSearch[]>([])
  smartFolders = $state<SmartFolder[]>([])
  smartFolderPages = $state<PageTreeNode[]>([])

  async loadSavedSearches() {
    this.savedSearches = await api.getAllSavedSearches()
  }

  async createSavedSearch(name: string, query: string, tagFilter: string | null, pinned: boolean): Promise<SavedSearch> {
    const search = await api.createSavedSearch({ name, query, tagFilter, pinned })
    await this.loadSavedSearches()
    return search
  }

  async updateSavedSearch(id: string, updates: { name?: string; query?: string; tagFilter?: string | null; pinned?: boolean }) {
    const search = await api.updateSavedSearch({ id, ...updates })
    await this.loadSavedSearches()
    return search
  }

  async deleteSavedSearch(id: string) {
    await api.deleteSavedSearch(id)
    await this.loadSavedSearches()
  }

  async executeSavedSearch(id: string): Promise<SearchResult[]> {
    return api.executeSavedSearch(id)
  }

  async loadSmartFolders() {
    this.smartFolders = await api.getAllSmartFolders()
  }

  async createSmartFolder(name: string, icon: string, rules: string): Promise<SmartFolder> {
    const folder = await api.createSmartFolder({ name, icon, rules })
    await this.loadSmartFolders()
    return folder
  }

  async updateSmartFolder(id: string, updates: { name?: string; icon?: string; rules?: string; sortOrder?: number }) {
    const folder = await api.updateSmartFolder({ id, ...updates })
    await this.loadSmartFolders()
    return folder
  }

  async deleteSmartFolder(id: string) {
    await api.deleteSmartFolder(id)
    await this.loadSmartFolders()
  }

  async loadSmartFolderPages(id: string) {
    this.smartFolderPages = await api.getSmartFolderPages(id)
  }
}

export class HistoryStore {
  revisions = $state<PageRevision[]>([])
  favorites = $state<Favorite[]>([])
  currentFavoriteIds = $state<Set<string>>(new Set())

  async loadRevisions(pageId: string) {
    this.revisions = await api.getPageRevisions(pageId)
  }

  async restoreRevision(revisionId: string): Promise<Page> {
    const page = await api.restoreRevision(revisionId)
    await this.loadRevisions(page.id)
    return page
  }

  async deleteRevision(id: string) {
    await api.deleteRevision(id)
    if (this.revisions.length > 0) {
      this.revisions = this.revisions.filter((r) => r.id !== id)
    }
  }

  async loadFavorites() {
    this.favorites = await api.getFavorites()
    this.currentFavoriteIds = new Set(this.favorites.map((f) => f.pageId))
  }

  async addFavorite(pageId: string) {
    const fav = await api.addFavorite(pageId)
    this.currentFavoriteIds.add(pageId)
    await this.loadFavorites()
    return fav
  }

  async removeFavorite(pageId: string) {
    await api.removeFavorite(pageId)
    this.currentFavoriteIds.delete(pageId)
    await this.loadFavorites()
  }

  async checkFavorite(pageId: string): Promise<boolean> {
    const isFav = await api.isFavorite(pageId)
    if (isFav) this.currentFavoriteIds.add(pageId)
    else this.currentFavoriteIds.delete(pageId)
    return isFav
  }

  async reorderFavorites(orderedPageIds: string[]) {
    await api.reorderFavorites(orderedPageIds)
    await this.loadFavorites()
  }
}

export class DiscoveryStore {
  tagGroups = $state<TagGroup[]>([])
  ungroupedTags = $state<Tag[]>([])
  relatedPages = $state<RelatedPage[]>([])

  async loadTagGroups() {
    this.tagGroups = await api.getAllTagGroups()
  }

  async createTagGroup(name: string, color: string | null): Promise<TagGroup> {
    const group = await api.createTagGroup({ name, color })
    await this.loadTagGroups()
    return group
  }

  async updateTagGroup(id: string, updates: { name?: string; color?: string | null; sortOrder?: number }) {
    const group = await api.updateTagGroup({ id, ...updates })
    await this.loadTagGroups()
    return group
  }

  async deleteTagGroup(id: string) {
    await api.deleteTagGroup(id)
    await this.loadTagGroups()
  }

  async addTagToGroup(groupId: string, tagId: string) {
    await api.addTagToGroup(groupId, tagId)
  }

  async removeTagFromGroup(groupId: string, tagId: string) {
    await api.removeTagFromGroup(groupId, tagId)
  }

  async loadUngroupedTags() {
    this.ungroupedTags = await api.getUngroupedTags()
  }

  async loadRelatedPages(pageId: string, limit?: number) {
    this.relatedPages = await api.getRelatedPages(pageId, limit)
  }
}

export class AttachmentStore {
  attachments = $state<Attachment[]>([])

  async loadAttachments(pageId: string) {
    this.attachments = await api.getAttachmentsForPage(pageId)
  }

  async createAttachment(pageId: string, fileName: string, mimeType: string, data: number[]): Promise<Attachment> {
    const attachment = await api.createAttachment({ pageId, fileName, mimeType, data })
    await this.loadAttachments(pageId)
    return attachment
  }

  async deleteAttachment(id: string, pageId: string) {
    await api.deleteAttachment(id)
    await this.loadAttachments(pageId)
  }

  async getAttachmentData(id: string): Promise<{ attachment: Attachment; data: number[] }> {
    return await api.getAttachment(id)
  }
}

export class SettingsStore {
  theme = $state<'light' | 'dark' | 'system' | 'high-contrast'>('system')
  language = $state('en')
  fontSize = $state(16)
  lineSpacing = $state(1.5)
  sidebarCollapsed = $state(false)
  activeTagFilter = $state<string | null>(null)
  sidebarTab = $state<string>('pages')

  async loadSettings() {
    const settings = await api.getAllSettings()
    for (const [key, value] of settings) {
      switch (key) {
        case 'theme':
          this.theme = value as 'light' | 'dark' | 'system' | 'high-contrast'
          break
        case 'language':
          this.language = value
          break
        case 'font_size':
          this.fontSize = parseInt(value, 10) || 16
          break
        case 'line_spacing':
          this.lineSpacing = parseFloat(value) || 1.5
          break
      }
    }
    this.applyTheme()
  }

  async setTheme(theme: 'light' | 'dark' | 'system' | 'high-contrast') {
    this.theme = theme
    await api.setSetting('theme', theme)
    this.applyTheme()
  }

  async setLanguage(lang: string) {
    this.language = lang
    await api.setSetting('language', lang)
  }

  async setFontSize(size: number) {
    this.fontSize = size
    await api.setSetting('font_size', String(size))
  }

  async setLineSpacing(spacing: number) {
    this.lineSpacing = spacing
    await api.setSetting('line_spacing', String(spacing))
  }

  applyTheme() {
    const root = document.documentElement
    root.classList.remove('dark', 'high-contrast')
    if (this.theme === 'dark') {
      root.classList.add('dark')
    } else if (this.theme === 'high-contrast') {
      root.classList.add('high-contrast')
    } else if (this.theme === 'light') {
      // neither class
    } else {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      root.classList.toggle('dark', prefersDark)
    }
  }
}

export class SyncStore {
  status = $state<SyncStatus | null>(null)
  syncing = $state(false)
  error = $state<string | null>(null)
  pendingCount = $state(0)

  async loadStatus() {
    try {
      this.status = await api.getSyncStatus()
      this.pendingCount = await api.getPendingSyncCount()
    } catch {
      this.status = null
    }
  }

  async start(deviceName: string | undefined, port: number | undefined, pairingSecret: string) {
    this.error = null
    try {
      this.status = await api.startSync(deviceName, port, pairingSecret)
    } catch (e) {
      this.error = String(e)
    }
  }

  async stop() {
    try {
      await api.stopSync()
      this.status = null
    } catch (e) {
      this.error = String(e)
    }
  }

  async syncWithPeer(peerId: string) {
    this.syncing = true
    this.error = null
    try {
      await api.syncWithPeer(peerId)
      await this.loadStatus()
    } catch (e) {
      this.error = String(e)
    }
    this.syncing = false
  }

  async applyCrdtToDb() {
    this.syncing = true
    this.error = null
    try {
      await api.applyCrdtToDb()
      await this.loadStatus()
    } catch (e) {
      this.error = String(e)
    }
    this.syncing = false
  }

  get peers(): SyncDeviceInfo[] {
    return this.status?.peers ?? []
  }

  get enabled(): boolean {
    return this.status?.enabled ?? false
  }

  get lastSync(): string | null {
    return this.status?.lastSync ?? null
  }
}

export class WorkspaceStore {
  workspaces = $state<Workspace[]>([])
  activeWorkspace = $state<Workspace | null>(null)
  isLoading = $state(false)

  async load() {
    this.isLoading = true
    try {
      this.workspaces = await api.listWorkspaces()
      this.activeWorkspace = await api.getActiveWorkspace()
    } finally {
      this.isLoading = false
    }
  }

  async create(name: string) {
    await api.createWorkspace(name)
    await this.load()
  }

  async remove(id: string) {
    await api.deleteWorkspace(id)
    await this.load()
  }

  async rename(id: string, name: string) {
    await api.renameWorkspace(id, name)
    await this.load()
  }

  async switch(id: string) {
    await api.switchWorkspace(id)
    await this.load()
    await pageStore.loadPageTree()
  }
}

export class EncryptionStore {
  enabled = $state(false)
  unlocked = $state(false)
  error = $state<string | null>(null)

  async checkStatus() {
    try {
      this.enabled = await api.isEncryptionEnabled()
      if (!this.enabled) {
        this.unlocked = true
      }
    } catch (e) {
      this.error = String(e)
    }
  }

  async enable(passphrase: string) {
    this.error = null
    try {
      await api.enableEncryption(passphrase)
      this.enabled = true
      this.unlocked = true
    } catch (e) {
      this.error = String(e)
      throw e
    }
  }

  async unlock(passphrase: string) {
    this.error = null
    try {
      const result = await api.unlockDatabase(passphrase)
      this.unlocked = result
      if (!result) {
        this.error = 'Invalid passphrase'
      }
      return result
    } catch (e) {
      this.error = String(e)
      throw e
    }
  }

  async disable(passphrase: string) {
    this.error = null
    try {
      await api.disableEncryption(passphrase)
      this.enabled = false
      this.unlocked = true
    } catch (e) {
      this.error = String(e)
      throw e
    }
  }

  async changePassphrase(oldPassphrase: string, newPassphrase: string) {
    this.error = null
    try {
      await api.changePassphrase(oldPassphrase, newPassphrase)
    } catch (e) {
      this.error = String(e)
      throw e
    }
  }
}

export const pageStore = new PageStore()
export const tagStore = new TagStore()
export const backlinkStore = new BacklinkStore()
export const propertyStore = new PropertyStore()
export const templateStore = new TemplateStore()
export const searchStore = new SearchStore()
export const historyStore = new HistoryStore()
export const discoveryStore = new DiscoveryStore()
export const attachmentStore = new AttachmentStore()
export const settingsStore = new SettingsStore()
export const syncStore = new SyncStore()
export const workspaceStore = new WorkspaceStore()
export const encryptionStore = new EncryptionStore()
