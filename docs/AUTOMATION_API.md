# Automation API

900Notes exposes a local IPC API via Tauri commands for scripting against the knowledge base. These commands can be invoked from the Tauri webview, plugins, or external scripts via the Tauri CLI.

## Pages

### Create Page

```typescript
const page = await invoke('api_create_page', {
  title: 'My Note',
  content: '{"type":"doc","content":[{"type":"paragraph"}]}',
  parentId: null,
});
```

### Get Page

```typescript
const page = await invoke('api_get_page', { id: 'page-uuid' });
```

### Update Page

```typescript
const page = await invoke('api_update_page', {
  id: 'page-uuid',
  title: 'Updated Title',
  content: '{"type":"doc","content":[...]}',
  icon: '📝',
  coverColor: '#2563eb',
  pinned: true,
});
```

### Delete Page

```typescript
await invoke('api_delete_page', { id: 'page-uuid' });
```

### Search Pages

```typescript
const results = await invoke('api_search_pages', { query: 'meeting notes', limit: 20 });
```

### Get All Pages (metadata only)

```typescript
const pages = await invoke('api_get_all_pages');
```

### Get Page Tree (metadata only)

```typescript
const tree = await invoke('api_get_page_tree');
```

### Get Recent Pages

```typescript
const recent = await invoke('api_get_recent_pages', { limit: 20 });
```

## Tags

### Create Tag

```typescript
const tag = await invoke('api_create_tag', { name: 'important', color: '#ef4444' });
```

### Get All Tags

```typescript
const tags = await invoke('api_get_all_tags');
```

### Set Page Tags

```typescript
await invoke('api_set_page_tags', { pageId: 'page-uuid', tagIds: ['tag-1', 'tag-2'] });
```

## Links

### Get Backlinks

```typescript
const backlinks = await invoke('api_get_backlinks', { pageId: 'page-uuid' });
```

## Settings

### Get Setting

```typescript
const value = await invoke('api_get_setting', { key: 'theme' });
```

### Set Setting

```typescript
await invoke('api_set_setting', { key: 'theme', value: 'dark' });
```

## Importers

### Evernote ENEX

```typescript
const result = await invoke('import_evernote', { enexContent: enexFileContent });
// result: { pagesCreated: number, errors: string[] }
```

### Notion Export

```typescript
const result = await invoke('import_notion', { dirPath: '/path/to/notion/export' });
```

### Obsidian Vault

```typescript
const result = await invoke('import_obsidian', { dirPath: '/path/to/vault' });
```

### Roam JSON

```typescript
const result = await invoke('import_roam', { jsonContent: roamJsonString });
```

## JavaScript API Wrappers

All automation commands are also available as typed wrappers in `src/lib/api.ts`:

```typescript
import * as api from './lib/api';

// Create a page
const page = await api.apiCreatePage('My Note', '{"type":"doc",...}');

// Search
const results = await api.apiSearchPages('meeting', 20);

// Import from Obsidian
const result = await api.importObsidian('/path/to/vault');
```

## Scripting Examples

### Bulk create pages from a CSV

```typescript
import { apiCreatePage } from './lib/api';

const csv = `title,content
Note 1,First note
Note 2,Second note`;

for (const line of csv.split('\n').slice(1)) {
  const [title, content] = line.split(',');
  await apiCreatePage(title, JSON.stringify({
    type: 'doc',
    content: [{ type: 'paragraph', content: [{ type: 'text', text: content }] }],
  }));
}
```

### Export all pages to JSON

```typescript
import { apiGetAllPages, apiGetPage } from './lib/api';

const pages = await apiGetAllPages();
const export = [];
for (const meta of pages) {
  const full = await apiGetPage(meta.id);
  export.push({ title: full.title, content: full.content });
}
console.log(JSON.stringify(export, null, 2));
```
