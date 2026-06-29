import { invoke } from '@tauri-apps/api/core'
import type {
  Page,
  PageMetadata,
  PageTreeNode,
  PageTreeNodeMeta,
  CreatePageInput,
  UpdatePageInput,
  MovePageInput,
  Tag,
  CreateTagInput,
  UpdateTagInput,
  SearchResult,
  Backlink,
  Link,
  PageProperty,
  SetPropertyInput,
  Template,
  CreateTemplateInput,
  UpdateTemplateInput,
  GraphData,
  SavedSearch,
  CreateSavedSearchInput,
  UpdateSavedSearchInput,
  SmartFolder,
  CreateSmartFolderInput,
  UpdateSmartFolderInput,
  PageRevision,
  Favorite,
  TagGroup,
  CreateTagGroupInput,
  UpdateTagGroupInput,
  RelatedPage,
  Attachment,
  CreateAttachmentInput,
  GetAttachmentResponse,
  AudioNote,
  CreateAudioNoteInput,
  UpdateAudioNoteInput,
  SyncStatus,
  SyncConflict,
  Workspace,
  Plugin,
  PluginManifest,
} from './types'

// ── Pages ──
export const createPage = (input: CreatePageInput): Promise<Page> =>
  invoke('create_page', { input })

export const getPage = (id: string): Promise<Page> =>
  invoke('get_page', { id })

export const getAllPages = (): Promise<Page[]> =>
  invoke('get_all_pages')

export const getPageTree = (): Promise<PageTreeNode[]> =>
  invoke('get_page_tree')

export const getPageTreeMetadata = (): Promise<PageTreeNodeMeta[]> =>
  invoke('get_page_tree_metadata')

export const getPageTitles = (): Promise<[string, string][]> =>
  invoke('get_page_titles')

export const getRecentPagesMetadata = (limit?: number): Promise<PageMetadata[]> =>
  invoke('get_recent_pages_metadata', { limit: limit ?? 10 })

export const updatePage = (input: UpdatePageInput): Promise<Page> =>
  invoke('update_page', { input })

export const deletePage = (id: string): Promise<void> =>
  invoke('delete_page', { id })

export const restorePage = (id: string): Promise<Page> =>
  invoke('restore_page', { id })

export const duplicatePage = (id: string): Promise<Page> =>
  invoke('duplicate_page', { id })

export const movePage = (input: MovePageInput): Promise<Page> =>
  invoke('move_page', { input })

export const getRecentPages = (limit?: number): Promise<Page[]> =>
  invoke('get_recent_pages', { limit: limit ?? 10 })

export const getTrash = (): Promise<Page[]> =>
  invoke('get_trash')

export const emptyTrash = (): Promise<void> =>
  invoke('empty_trash')

export const secureDeletePage = (id: string): Promise<void> =>
  invoke('secure_delete_page', { id })

export const secureEmptyTrash = (): Promise<void> =>
  invoke('secure_empty_trash')

export const searchPages = (query: string, limit?: number): Promise<SearchResult[]> =>
  invoke('search_pages', { query, limit: limit ?? 50 })

// ── Tags ──
export const getAllTags = (): Promise<Tag[]> =>
  invoke('get_all_tags')

export const createTag = (input: CreateTagInput): Promise<Tag> =>
  invoke('create_tag', { input })

export const updateTag = (input: UpdateTagInput): Promise<Tag> =>
  invoke('update_tag', { input })

export const deleteTag = (id: string): Promise<void> =>
  invoke('delete_tag', { id })

export const getPageTags = (pageId: string): Promise<Tag[]> =>
  invoke('get_page_tags', { pageId })

export const setPageTags = (pageId: string, tagIds: string[]): Promise<void> =>
  invoke('set_page_tags', { pageId, tagIds })

// ── Links ──
export const getBacklinks = (pageId: string): Promise<Backlink[]> =>
  invoke('get_backlinks', { pageId })

export const getOutgoingLinks = (pageId: string): Promise<Link[]> =>
  invoke('get_outgoing_links', { pageId })

export const rebuildLinks = (): Promise<void> =>
  invoke('rebuild_links')

// ── Settings ──
export const getSetting = (key: string): Promise<string | null> =>
  invoke('get_setting', { key })

export const setSetting = (key: string, value: string): Promise<void> =>
  invoke('set_setting', { key, value })

export const getAllSettings = (): Promise<[string, string][]> =>
  invoke('get_all_settings')

// ── Export/Import ──
export const exportWorkspace = (): Promise<string> =>
  invoke('export_workspace')

export const importWorkspace = (data: string): Promise<number> =>
  invoke('import_workspace', { data })

export const exportPageMarkdown = (pageId: string): Promise<string> =>
  invoke('export_page_markdown', { pageId })

export const importMarkdown = (
  title: string,
  content: string,
  parentId: string | null,
): Promise<Page> =>
  invoke('import_markdown', { title, content, parentId })

// ── Page Properties ──
export const getPageProperties = (pageId: string): Promise<PageProperty[]> =>
  invoke('get_page_properties', { pageId })

export const setPageProperty = (input: SetPropertyInput): Promise<PageProperty> =>
  invoke('set_page_property', { input })

export const deletePageProperty = (id: string): Promise<void> =>
  invoke('delete_page_property', { id })

// ── Templates ──
export const getAllTemplates = (): Promise<Template[]> =>
  invoke('get_all_templates')

export const createTemplate = (input: CreateTemplateInput): Promise<Template> =>
  invoke('create_template', { input })

export const updateTemplate = (input: UpdateTemplateInput): Promise<Template> =>
  invoke('update_template', { input })

export const deleteTemplate = (id: string): Promise<void> =>
  invoke('delete_template', { id })

export const createPageFromTemplate = (
  templateId: string,
  title: string,
  parentId: string | null,
): Promise<Page> =>
  invoke('create_page_from_template', { templateId, title, parentId })

export const getOrCreateDailyNote = (date: string): Promise<Page> =>
  invoke('get_or_create_daily_note', { date })

// ── Graph ──
export const getGraphData = (): Promise<GraphData> =>
  invoke('get_graph_data')

// ── Saved Searches ──
export const getAllSavedSearches = (): Promise<SavedSearch[]> =>
  invoke('get_all_saved_searches')

export const createSavedSearch = (input: CreateSavedSearchInput): Promise<SavedSearch> =>
  invoke('create_saved_search', { input })

export const updateSavedSearch = (input: UpdateSavedSearchInput): Promise<SavedSearch> =>
  invoke('update_saved_search', { input })

export const deleteSavedSearch = (id: string): Promise<void> =>
  invoke('delete_saved_search', { id })

export const executeSavedSearch = (id: string): Promise<SearchResult[]> =>
  invoke('execute_saved_search', { id })

// ── Smart Folders ──
export const getAllSmartFolders = (): Promise<SmartFolder[]> =>
  invoke('get_all_smart_folders')

export const createSmartFolder = (input: CreateSmartFolderInput): Promise<SmartFolder> =>
  invoke('create_smart_folder', { input })

export const updateSmartFolder = (input: UpdateSmartFolderInput): Promise<SmartFolder> =>
  invoke('update_smart_folder', { input })

export const deleteSmartFolder = (id: string): Promise<void> =>
  invoke('delete_smart_folder', { id })

export const getSmartFolderPages = (id: string): Promise<PageTreeNode[]> =>
  invoke('get_smart_folder_pages', { id })

// ── Page Revisions ──
export const getPageRevisions = (pageId: string): Promise<PageRevision[]> =>
  invoke('get_page_revisions', { pageId })

export const getRevision = (id: string): Promise<PageRevision> =>
  invoke('get_revision', { id })

export const restoreRevision = (revisionId: string): Promise<Page> =>
  invoke('restore_revision', { revisionId })

export const deleteRevision = (id: string): Promise<void> =>
  invoke('delete_revision', { id })

// ── Favorites ──
export const getFavorites = (): Promise<Favorite[]> =>
  invoke('get_favorites')

export const addFavorite = (pageId: string): Promise<Favorite> =>
  invoke('add_favorite', { pageId })

export const removeFavorite = (pageId: string): Promise<void> =>
  invoke('remove_favorite', { pageId })

export const isFavorite = (pageId: string): Promise<boolean> =>
  invoke('is_favorite', { pageId })

export const reorderFavorites = (orderedPageIds: string[]): Promise<void> =>
  invoke('reorder_favorites', { orderedPageIds })

// ── Tag Groups ──
export const getAllTagGroups = (): Promise<TagGroup[]> =>
  invoke('get_all_tag_groups')

export const createTagGroup = (input: CreateTagGroupInput): Promise<TagGroup> =>
  invoke('create_tag_group', { input })

export const updateTagGroup = (input: UpdateTagGroupInput): Promise<TagGroup> =>
  invoke('update_tag_group', { input })

export const deleteTagGroup = (id: string): Promise<void> =>
  invoke('delete_tag_group', { id })

export const addTagToGroup = (groupId: string, tagId: string): Promise<void> =>
  invoke('add_tag_to_group', { groupId, tagId })

export const removeTagFromGroup = (groupId: string, tagId: string): Promise<void> =>
  invoke('remove_tag_from_group', { groupId, tagId })

export const getTagsInGroup = (groupId: string): Promise<Tag[]> =>
  invoke('get_tags_in_group', { groupId })

export const getUngroupedTags = (): Promise<Tag[]> =>
  invoke('get_ungrouped_tags')

// ── Related Pages ──
export const getRelatedPages = (pageId: string, limit?: number): Promise<RelatedPage[]> =>
  invoke('get_related_pages', { pageId, limit })

// ── Attachments ──
export const createAttachment = (input: CreateAttachmentInput): Promise<Attachment> =>
  invoke('create_attachment', { input })

export const getAttachment = (id: string): Promise<GetAttachmentResponse> =>
  invoke('get_attachment', { id })

export const getAttachmentsForPage = (pageId: string): Promise<Attachment[]> =>
  invoke('get_attachments_for_page', { pageId })

export const deleteAttachment = (id: string): Promise<void> =>
  invoke('delete_attachment', { id })

// ── PDF Export ──
export const exportPagePdf = (pageId: string): Promise<number[]> =>
  invoke('export_page_pdf', { pageId })

export const exportWorkspacePdf = (): Promise<number[]> =>
  invoke('export_workspace_pdf')

// ── OCR ──
export const ocrAttachment = (attachmentId: string): Promise<string> =>
  invoke('ocr_attachment', { attachmentId })

// ── Audio Notes ──
export const createAudioNote = (input: CreateAudioNoteInput): Promise<AudioNote> =>
  invoke('create_audio_note', { input })

export const getAudioNote = (id: string): Promise<AudioNote> =>
  invoke('get_audio_note', { id })

export const getAudioNotesForPage = (pageId: string): Promise<AudioNote[]> =>
  invoke('get_audio_notes_for_page', { pageId })

export const updateAudioNote = (input: UpdateAudioNoteInput): Promise<AudioNote> =>
  invoke('update_audio_note', { input })

export const deleteAudioNote = (id: string): Promise<void> =>
  invoke('delete_audio_note', { id })

// ── Sync ──
export const startSync = (deviceName?: string, port?: number): Promise<SyncStatus> =>
  invoke('start_sync', { deviceName, port })

export const stopSync = (): Promise<void> =>
  invoke('stop_sync')

export const getSyncStatus = (): Promise<SyncStatus> =>
  invoke('get_sync_status')

export const syncWithPeer = (peerId: string): Promise<SyncConflict[]> =>
  invoke('sync_with_peer', { peerId })

export const syncPageToCrdt = (pageId: string): Promise<void> =>
  invoke('sync_page_to_crdt', { pageId })

export const getPendingSyncCount = (): Promise<number> =>
  invoke('get_pending_sync_count')

export const applyCrdtToDb = (): Promise<number> =>
  invoke('apply_crdt_to_db')

// ── Sharing ──
export const exportShareBundle = (pageIds: string[], passphrase: string): Promise<string> =>
  invoke('export_share_bundle', { pageIds, passphrase })

export const importShareBundle = (encryptedData: string, passphrase: string): Promise<number> =>
  invoke('import_share_bundle', { encryptedData, passphrase })

// ── HTML Export ──
export const exportPageHtml = (pageId: string): Promise<string> =>
  invoke('export_page_html', { pageId })

export const exportPagesHtml = (pageIds: string[]): Promise<string> =>
  invoke('export_pages_html', { pageIds })

// ── Workspaces ──
export const listWorkspaces = (): Promise<Workspace[]> =>
  invoke('list_workspaces')

export const getActiveWorkspace = (): Promise<Workspace> =>
  invoke('get_active_workspace')

export const createWorkspace = (name: string): Promise<Workspace> =>
  invoke('create_workspace', { name })

export const deleteWorkspace = (id: string): Promise<void> =>
  invoke('delete_workspace', { id })

export const renameWorkspace = (id: string, name: string): Promise<Workspace> =>
  invoke('rename_workspace', { id, name })

export const switchWorkspace = (id: string): Promise<string> =>
  invoke('switch_workspace', { id })

// ── Encryption ──
export const isEncryptionEnabled = (): Promise<boolean> =>
  invoke('is_encryption_enabled')

export const enableEncryption = (passphrase: string): Promise<void> =>
  invoke('enable_encryption', { passphrase })

export const unlockDatabase = (passphrase: string): Promise<boolean> =>
  invoke('unlock_database', { passphrase })

export const disableEncryption = (passphrase: string): Promise<void> =>
  invoke('disable_encryption', { passphrase })

export const changePassphrase = (oldPassphrase: string, newPassphrase: string): Promise<void> =>
  invoke('change_passphrase', { oldPassphrase, newPassphrase })

export const verifyPassphrase = (passphrase: string): Promise<boolean> =>
  invoke('verify_passphrase', { passphrase })

export const exportEncryptedWorkspace = (passphrase: string): Promise<string> =>
  invoke('export_encrypted_workspace', { passphrase })

export const importEncryptedWorkspace = (encryptedData: string, passphrase: string): Promise<number> =>
  invoke('import_encrypted_workspace', { encryptedData, passphrase })

export const getAllPlugins = (): Promise<Plugin[]> =>
  invoke('get_all_plugins')

export const getEnabledPlugins = (): Promise<Plugin[]> =>
  invoke('get_enabled_plugins')

export const installPlugin = (manifest: PluginManifest): Promise<Plugin> =>
  invoke('install_plugin', { manifest })

export const setPluginEnabled = (id: string, enabled: boolean): Promise<void> =>
  invoke('set_plugin_enabled', { id, enabled })

export const uninstallPlugin = (id: string): Promise<void> =>
  invoke('uninstall_plugin', { id })

export const scanPluginsDir = (appDataDir: string): Promise<Plugin[]> =>
  invoke('scan_plugins_dir', { appDataDir })

export const readPluginFile = (appDataDir: string, pluginId: string, filePath: string): Promise<string> =>
  invoke('read_plugin_file', { appDataDir, pluginId, filePath })

// ── Automation API ──
export const apiCreatePage = (title: string, content?: string, parentId?: string): Promise<Page> =>
  invoke('api_create_page', { title, content, parentId })

export const apiGetPage = (id: string): Promise<Page> =>
  invoke('api_get_page', { id })

export const apiUpdatePage = (
  id: string,
  title?: string,
  content?: string,
  icon?: string,
  coverColor?: string,
  pinned?: boolean,
): Promise<Page> =>
  invoke('api_update_page', { id, title, content, icon, coverColor, pinned })

export const apiDeletePage = (id: string): Promise<void> =>
  invoke('api_delete_page', { id })

export const apiSearchPages = (query: string, limit?: number): Promise<SearchResult[]> =>
  invoke('api_search_pages', { query, limit })

export const apiGetAllPages = (): Promise<PageMetadata[]> =>
  invoke('api_get_all_pages')

export const apiGetPageTree = (): Promise<PageTreeNodeMeta[]> =>
  invoke('api_get_page_tree')

export const apiGetRecentPages = (limit?: number): Promise<PageMetadata[]> =>
  invoke('api_get_recent_pages', { limit })

export const apiCreateTag = (name: string, color?: string): Promise<Tag> =>
  invoke('api_create_tag', { name, color })

export const apiGetAllTags = (): Promise<Tag[]> =>
  invoke('api_get_all_tags')

export const apiSetPageTags = (pageId: string, tagIds: string[]): Promise<void> =>
  invoke('api_set_page_tags', { pageId, tagIds })

export const apiGetBacklinks = (pageId: string): Promise<Backlink[]> =>
  invoke('api_get_backlinks', { pageId })

export const apiGetSetting = (key: string): Promise<string | null> =>
  invoke('api_get_setting', { key })

export const apiSetSetting = (key: string, value: string): Promise<void> =>
  invoke('api_set_setting', { key, value })

// ── Importers ──
export interface ImportResult {
  pagesCreated: number
  errors: string[]
}

export const importEvernote = (enexContent: string): Promise<ImportResult> =>
  invoke('import_evernote', { enexContent })

export const importNotion = (dirPath: string): Promise<ImportResult> =>
  invoke('import_notion', { dirPath })

export const importObsidian = (dirPath: string): Promise<ImportResult> =>
  invoke('import_obsidian', { dirPath })

export const importRoam = (jsonContent: string): Promise<ImportResult> =>
  invoke('import_roam', { jsonContent })
