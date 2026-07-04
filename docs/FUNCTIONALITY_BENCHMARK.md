# Functionality Benchmark

Date: 2026-07-02

This benchmark compares 900Notes against current patterns in adjacent note and knowledge-base apps. It focuses on the app shell, menus, capture, search, graph navigation, and workspace review workflows.

## Sources

- [Obsidian Graph view](https://obsidian.md/help/plugins/graph) and [Command palette](https://obsidian.md/help/plugins/command-palette)
- [Notion database views](https://www.notion.com/help/guides/using-database-views) and [Buttons](https://www.notion.com/help/buttons)
- [Joplin Web Clipper](https://joplinapp.org/help/apps/clipper/) and [Search](https://joplinapp.org/help/apps/search/)
- [Capacities Collections](https://docs.capacities.io/reference/collections) and [Queries](https://docs.capacities.io/reference/queries)
- [Anytype Graph](https://doc.anytype.io/anytype-docs/advanced/feature-list-by-platform/graph)

## Competitive Patterns

| Pattern | Similar-app baseline | 900Notes status |
| --- | --- | --- |
| Fast command access | Obsidian exposes commands from the keyboard with shortcuts and fuzzy matching. Joplin has fast search syntax and "Goto Anything" behavior. | Stronger after this pass: command palette now runs commands, searches pages, jumps by title with `@`, filters tags with `#`, and can save a query from normal search text. |
| Quick capture | Joplin's clipper captures pages and screenshots into the desktop app. Notion uses buttons to reduce repetitive creation flows. | Stronger: Quick capture creates Inbox items with body text and tags, and Web capture stores source URLs, excerpts, tags, and capture metadata. The example browser extension now posts to a localhost clipper endpoint. Still missing a published extension, screenshot capture, and system share target. |
| Graph navigation | Obsidian and Anytype make graph views first-class, with global graph controls, color/group filters, local graph context, and node actions. | Stronger after this pass: global and Local graph include search, pan/zoom, depth filtering, filtered edge counts, color-by modes, a color legend, node selection, neighbor highlighting, and node actions for center/open. Still missing persistent layouts, richer tag/group clustering, and a minimap. |
| Smart views | Notion database views and Capacities queries make filtered, grouped, reusable views central to workspace organization. | Stronger after this pass: saved searches and smart folders exist, the palette can create saved searches, and the dashboard now embeds live smart-view previews with result counts and page shortcuts. Still missing richer grouping and property-based table/board layouts. |
| Workspace review | Mature apps surface unlinked content, tags, backlinks, and graph health as ongoing maintenance paths. | Improved: dashboard now includes Review queue sections for orphan pages, untagged pages, and hubs. |
| Menus | Comparable desktop apps keep creation, navigation, view, export, and tools actions discoverable from menus and command search. | Stronger: Create, Navigate, View, Export, and Tools actions are available in the in-app menu and the native desktop menu, with key capture and command-palette shortcuts wired through the same command dispatcher. Remaining gap: dynamic native menu enable/disable state for page-specific actions. |
| Automation and templates | Notion buttons and templates make repeated workflows feel less manual. | Partial: templates exist and Quick capture reduces creation friction. Missing configurable buttons, page actions, and reusable workflow commands. |
| Import/export and portability | Joplin emphasizes clipper, import/export, and local API surfaces. | Stronger: import/backup, Markdown/PDF export, sharing bundles, sync, plugin settings, typed automation, and localhost browser clipping are present. The remaining portability gap is packaging/signing rather than the intake path itself. |

## Changes Landed From This Review

- Added Quick capture as a real modal workflow, shortcut, command-palette action, dashboard action, and menu item.
- Added Web capture as a dedicated Quick Capture mode, command-palette action, dashboard action, menu item, shortcut, and automation intake command.
- Added a localhost web clipper endpoint and wired the example browser extension to save page and selection clips into Inbox.
- Expanded the command palette into a mixed launcher for commands, page-title jumps, tag filters, saved searches, and full-text search results.
- Added Local graph from the editor and menu, with depth filtering and panel close.
- Added a workspace Review queue for orphan pages, untagged pages, and linked hubs.
- Hid page-specific command-palette actions when no page is selected, avoiding silent no-op commands from the dashboard.
- Added a native desktop menu that routes Create, Navigate, View, Export, and Tools actions into the same command dispatcher as the in-app menu and command palette.
- Added graph color modes, a visible legend, selected-node inspection, neighbor highlighting, and node actions so graph navigation is exploratory rather than click-to-open only.
- Added dashboard-embedded Smart views with live saved-search and smart-folder previews, result counts, and page shortcuts; fixed tag-rule smart folders so default smart-folder rules execute correctly.

## Remaining Benchmark Gaps

1. Packaged browser/system capture: publish/sign the browser extension, add screenshot capture, and add a platform share target.
2. Graph maturity: add persistent layouts, minimap/navigation aids, richer tag/group clustering, and graph filter presets.
3. Smart view maturity: add property-based table/board layouts, grouped smart views, and richer smart-folder rule editing.
4. Workflow buttons: add Notion-style configurable page buttons for repeatable actions.
5. Native desktop polish: add dynamic native menu enable/disable state, a standard Window menu, and deeper platform-specific polish.
