export interface Page {
  id: string
  parentId: string | null
  title: string
  content: string
  icon: string | null
  coverColor: string | null
  createdAt: string
  updatedAt: string
  deletedAt: string | null
  pinned: boolean
  sortOrder: number
}

export interface PageMetadata {
  id: string
  parentId: string | null
  title: string
  icon: string | null
  coverColor: string | null
  createdAt: string
  updatedAt: string
  deletedAt: string | null
  pinned: boolean
  sortOrder: number
}

export interface PageTreeNode {
  page: Page
  children: PageTreeNode[]
}

export interface PageTreeNodeMeta {
  page: PageMetadata
  children: PageTreeNodeMeta[]
}

export interface CreatePageInput {
  parentId: string | null
  title: string
  content?: string
  icon?: string | null
}

export interface UpdatePageInput {
  id: string
  title?: string
  content?: string
  icon?: string | null
  coverColor?: string | null
  pinned?: boolean
}

export interface MovePageInput {
  id: string
  parentId: string | null
  sortOrder: number
}

export interface Tag {
  id: string
  name: string
  color: string | null
  createdAt: string
}

export interface CreateTagInput {
  name: string
  color?: string | null
}

export interface UpdateTagInput {
  id: string
  name?: string
  color?: string | null
}

export interface SearchResult {
  id: string
  title: string
  snippet: string
  icon: string | null
  updatedAt: string
}

export interface Backlink {
  id: string
  sourcePageId: string
  sourcePageTitle: string
  sourcePageIcon: string | null
  linkText: string
  createdAt: string
}

export interface Link {
  id: string
  sourcePageId: string
  targetPageId: string
  linkText: string
  createdAt: string
}

export interface PageProperty {
  id: string
  pageId: string
  key: string
  value: string
  sortOrder: number
}

export interface SetPropertyInput {
  pageId: string
  key: string
  value: string
}

export interface Template {
  id: string
  name: string
  icon: string
  content: string
  category: string
  createdAt: string
  updatedAt: string
}

export interface CreateTemplateInput {
  name: string
  icon: string
  content: string
  category: string
}

export interface UpdateTemplateInput {
  id: string
  name?: string
  icon?: string
  content?: string
  category?: string
}

export interface GraphNode {
  id: string
  title: string
  icon: string | null
  tagCount: number
  linkCount: number
  createdAt: string
  updatedAt: string
}

export interface GraphEdge {
  source: string
  target: string
}

export interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export interface SavedSearch {
  id: string
  name: string
  query: string
  tagFilter: string | null
  pinned: boolean
  createdAt: string
  updatedAt: string
}

export interface CreateSavedSearchInput {
  name: string
  query: string
  tagFilter: string | null
  pinned: boolean
}

export interface UpdateSavedSearchInput {
  id: string
  name?: string
  query?: string
  tagFilter?: string | null
  pinned?: boolean
}

export interface SmartFolder {
  id: string
  name: string
  icon: string
  rules: string
  sortOrder: number
  createdAt: string
  updatedAt: string
}

export interface CreateSmartFolderInput {
  name: string
  icon: string
  rules: string
}

export interface UpdateSmartFolderInput {
  id: string
  name?: string
  icon?: string
  rules?: string
  sortOrder?: number
}

export interface SmartFolderRule {
  field: string
  operator: string
  value: string
}

export interface PageRevision {
  id: string
  pageId: string
  title: string
  content: string
  createdAt: string
}

export interface Favorite {
  id: string
  pageId: string
  sortOrder: number
  createdAt: string
}

export interface TagGroup {
  id: string
  name: string
  color: string | null
  sortOrder: number
  createdAt: string
  updatedAt: string
}

export interface CreateTagGroupInput {
  name: string
  color: string | null
}

export interface UpdateTagGroupInput {
  id: string
  name?: string
  color?: string | null
  sortOrder?: number
}

export interface RelatedPage {
  id: string
  title: string
  icon: string | null
  score: number
  reasons: string[]
}

export interface Attachment {
  id: string
  pageId: string
  fileName: string
  mimeType: string
  fileSize: number
  isImage: boolean
  createdAt: string
}

export interface CreateAttachmentInput {
  pageId: string
  fileName: string
  mimeType: string
  data: number[]
}

export interface GetAttachmentResponse {
  attachment: Attachment
  data: number[]
}

export interface AudioNote {
  id: string
  pageId: string
  attachmentId: string
  durationSec: number
  title: string
  transcription: string | null
  createdAt: string
}

export interface CreateAudioNoteInput {
  pageId: string
  attachmentId: string
  durationSec: number
  title: string
}

export interface UpdateAudioNoteInput {
  id: string
  title?: string
  transcription?: string
}

export interface SyncDeviceInfo {
  id: string
  name: string
  host: string
  port: number
}

export interface SyncStatus {
  enabled: boolean
  deviceId: string
  deviceName: string
  port: number
  peers: SyncDeviceInfo[]
  lastSync: string | null
}

export interface SyncConflict {
  pageId: string
  localUpdated: string
  remoteUpdated: string
  localTitle: string
  remoteTitle: string
}

export interface Workspace {
  id: string
  name: string
  dbPath: string
  createdAt: string
  isDefault: boolean
}

export interface Plugin {
  id: string
  name: string
  version: string
  author: string
  description: string
  enabled: boolean
  entryPoint: string
  installedAt: string
  updatedAt: string
}

export interface CustomBlockDef {
  nodeType: string
  group: string
  content: string | null
  attrs: Record<string, unknown>
  toDom: string
  parseDom: string | null
  icon: string | null
  label: string
}

export interface PluginManifest {
  id: string
  name: string
  version: string
  author: string
  description: string
  entryPoint: string
  customBlocks: CustomBlockDef[]
}

export interface PluginApi {
  registerBlock: (def: CustomBlockDef) => void
  registerCommand: (id: string, label: string, handler: () => void) => void
  registerHook: (event: string, handler: (...args: unknown[]) => void) => void
}

export interface LoadedPlugin {
  manifest: PluginManifest
  api: PluginApi
  blocks: CustomBlockDef[]
  commands: { id: string; label: string; handler: () => void }[]
  hooks: Map<string, ((...args: unknown[]) => void)[]>
}
