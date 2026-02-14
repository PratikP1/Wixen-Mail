# Rich HTML Rendering Pipeline + Attachment Preview/Open Requirements

## Objective

Implement a safer, richer message preview pipeline for HTML emails and provide an accessible attachment preview/open workflow in the integrated UI.

## Scope

1. Rich HTML rendering pipeline
   - sanitize unsafe HTML
   - generate accessible plain-text preview
   - extract safe links and image alt text
   - emit renderer warnings for security/accessibility
2. Attachment preview/open workflow
   - in-message attachment actions: Preview, Open, Save
   - preview dialog with metadata and accessible text
   - open attachment placeholder via OS-default app when payload bytes are unavailable

## Functional Requirements

### FR1: Rich HTML rendering
- `HtmlRenderer` must:
  - sanitize incoming HTML (script/event stripping)
  - generate plain text for screen-reader-friendly display
  - extract and expose safe links (`http`, `https`, `mailto`)
  - extract image `alt` texts
  - return warnings for stripped unsafe content and accessibility gaps

### FR2: Preview model
- Render output must include:
  - sanitized html
  - plain text
  - has_images / has_links
  - extracted links
  - extracted image alt texts
  - warning list

### FR3: Integrated message preview behavior
- When message body is loaded:
  - detect HTML-like content and render via `HtmlRenderer`
  - fallback to escaped/plain rendering for non-HTML content
- Message preview pane must show:
  - plain text
  - warnings section (if any)
  - clickable safe links

### FR4: Attachment actions
- Each attachment row in preview pane must expose:
  - Save
  - Preview
  - Open
- Preview opens an in-app dialog with attachment metadata/details.
- Open writes a placeholder preview file (when bytes unavailable) and launches default OS app.

### FR5: Security and filesystem safety
- Attachment open/save paths must:
  - sanitize file names for temp paths
  - avoid path traversal
- Link handling must reject unsafe schemes.

## Non-Functional Requirements

- Accessibility:
  - clear headings/labels for warnings, links, and preview details
  - text-first preview behavior
- Performance:
  - precompile regexes used repeatedly by renderer
- Backward compatibility:
  - keep existing plain-text rendering behavior functional

## Acceptance Criteria

1. New requirements doc exists.
2. `HtmlRenderer` exposes richer rendering metadata and safe link filtering.
3. `ui_integrated` message pane renders warnings/links for HTML content.
4. Attachments support Preview/Open/Save actions with useful feedback.
5. New/updated tests for renderer behavior pass.
6. Full test suite passes.
