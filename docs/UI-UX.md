# Facet UI/UX Specification

**Version:** 0.1

---

## Design Philosophy

Desktop-first interface optimized for fast, efficient data entry in a retail jewelry environment. The design should feel **premium and trustworthy** - reflecting the high-value items being handled.

---

## Themes

Two parallel themes will be developed to find the right aesthetic. Both share the same component structure and can be switched at runtime.

### Theme A: "Imperial" (Primary)

Clean, sophisticated American luxury aesthetic. Think high-end jewelry store display cases.

**Color Palette:**
| Role | Color | Hex |
|------|-------|-----|
| Background | Warm white | `#FAFAF8` |
| Surface | Pure white | `#FFFFFF` |
| Primary | Deep navy | `#1E3A5F` |
| Primary hover | Darker navy | `#152A45` |
| Accent | Muted gold | `#C9A227` |
| Text primary | Charcoal | `#2D2D2D` |
| Text secondary | Warm gray | `#6B6B6B` |
| Success | Forest green | `#2D5A3D` |
| Warning | Amber | `#D4A017` |
| Error/Overdue | Deep red | `#8B2635` |
| Rush badge | Gold | `#C9A227` |

**Typography:**
- Headings: Serif font (e.g., Playfair Display, Cormorant)
- Body: Clean sans-serif (e.g., Inter, DM Sans)
- Monospace (ticket codes): JetBrains Mono or similar

**Visual Elements:**
- Subtle shadows, no harsh borders
- Rounded corners (4-8px)
- Gold accents for premium feel
- Minimal iconography, text-forward

---

### Theme B: "Retro" (Alternative)

Playful, warm, vintage-inspired. Think 1970s jewelry shop meets modern usability.

**Color Palette:**
| Role | Color | Hex |
|------|-------|-----|
| Background | Warm cream | `#F5E6D3` |
| Surface | Off-white | `#FDF8F3` |
| Primary | Burnt orange | `#C85A2E` |
| Primary hover | Darker orange | `#A04725` |
| Accent | Teal | `#2A7B7B` |
| Text primary | Dark brown | `#3D2E1E` |
| Text secondary | Warm brown | `#7A6552` |
| Success | Olive green | `#5B7A3D` |
| Warning | Mustard | `#D4A017` |
| Error/Overdue | Terracotta | `#B84A3C` |
| Rush badge | Teal | `#2A7B7B` |

**Typography:**
- Headings: Retro-inspired (e.g., Fraunces, Recoleta)
- Body: Rounded sans-serif (e.g., Nunito, Quicksand)
- Monospace: IBM Plex Mono

**Visual Elements:**
- Slightly heavier shadows
- More rounded corners (8-12px)
- Warmer, cozier feel
- Can incorporate subtle patterns or textures

---

## Layout Structure

```
┌─────────────────────────────────────────────────────────────┐
│  Header: Store name, search bar, settings gear             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Main Content Area                                          │
│  (Workboard, Ticket Detail, Intake Form, etc.)              │
│                                                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

- **No sidebar navigation** - primary views accessed via header or workboard
- **Full-width workboard** - maximize horizontal space for status lanes
- **Modal overlays** for ticket detail, intake form, employee ID entry

---

## Screens & Interactions

### 1. Workboard (Home)

The primary view. Kanban-style board with status lanes.

```
┌──────────────────────────────────────────────────────────────────────┐
│  [Store Name]                    [🔍 Search...]         [⚙️ Settings] │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─ Intake ──┐  ┌─ In Progress ─┐  ┌─ Waiting ──┐  ┌─ Ready ────┐   │
│  │           │  │               │  │            │  │            │   │
│  │  [Card]   │  │   [Card]      │  │  [Card]    │  │  [Card]    │   │
│  │  [Card]   │  │   [Card]      │  │            │  │  [Card]    │   │
│  │           │  │   [Card]      │  │            │  │            │   │
│  │  + New    │  │               │  │            │  │            │   │
│  └───────────┘  └───────────────┘  └────────────┘  └────────────┘   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

**Lane behavior:**
- Lanes are fixed columns: Intake, In Progress, Waiting on Parts, Ready for Pickup
- Closed/Archived not shown (access via search)
- Each lane shows count in header
- Cards sorted: Rush first (with badge), then FIFO

**Card display:**
- Ticket code (prominent, e.g., "JR-0042")
- Customer name
- Item description (truncated)
- Thumbnail (first photo)
- Rush badge (if applicable)
- Overdue indicator (red border/background if past promise date)
- Promise date (if set)

**Card interactions:**
- **Click** → Opens ticket detail modal
- **Drag** → Move to different lane (triggers status change)
- Drop zones highlight on drag

**Drag-and-drop:**
- Visual feedback: card lifts with shadow, drop zone highlights
- On drop: prompt for Employee ID, then update status
- Animation: smooth transition to new lane position

**"+ New" button:**
- Located in Intake lane
- Opens intake form modal

---

### 2. Ticket Detail (Modal)

Full ticket information in a large modal overlay.

```
┌──────────────────────────────────────────────────────────────┐
│  JR-0042                                        [X] Close    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─ Customer ────────────────────────────────────────────┐   │
│  │ Jane Doe  •  555-1234  •  jane@example.com            │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─ Item ────────────────────────────────────────────────┐   │
│  │ Type: Ring                                            │   │
│  │ Description: Gold band with 0.5ct diamond             │   │
│  │ Condition: Minor scratches on band, prongs intact     │   │
│  │ Requested Work: Resize 7→6, polish, check prongs      │   │
│  │ Location: Safe Drawer 1                               │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─ Photos ──────────────────────────────────────────────┐   │
│  │ [img] [img] [img]  [+ Add Photo]                      │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─ Pricing ─────────────────────────────────────────────┐   │
│  │ Quote: $150.00          Actual: $145.00               │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─ Status ──────────────────────────────────────────────┐   │
│  │ Current: In Progress    [Rush: OFF]                   │   │
│  │ Promise Date: Jan 25, 2026 (3 days)                   │   │
│  │                                                       │   │
│  │ History:                                              │   │
│  │   • Intake → In Progress (Bob, Jan 19 2:00 PM)        │   │
│  │   • Created (Alice, Jan 19 10:30 AM)                  │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─ Notes ───────────────────────────────────────────────┐   │
│  │ [Add note field...]                          [Add]    │   │
│  │                                                       │   │
│  │ "Customer mentioned sentimental value" - Alice, 10:35 │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─ Actions ─────────────────────────────────────────────┐   │
│  │ [Edit Ticket]  [Print Receipt]  [Print Tag]  [Close]  │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Interactions:**
- Edit opens inline editing or sub-modal
- Print opens PDF in new tab
- Close Ticket button (only in Ready for Pickup status) prompts for actual amount, then employee ID
- Rush toggle prompts for employee ID

---

### 3. Intake Form (Modal)

Single scrolling form for new ticket creation.

```
┌──────────────────────────────────────────────────────────────┐
│  New Repair Ticket                              [X] Cancel   │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ── Customer ──────────────────────────────────────────────  │
│  Name*        [________________________] [🔍 existing]       │
│  Phone        [________________________]                     │
│  Email        [________________________]                     │
│                                                              │
│  ── Item ──────────────────────────────────────────────────  │
│  Type         [________________________] (optional)          │
│  Description* [________________________]                     │
│  Condition*   [                                          ]   │
│               [                                          ]   │
│  Requested    [                                          ]   │
│  Work*        [                                          ]   │
│                                                              │
│  ── Details ───────────────────────────────────────────────  │
│  Storage*     [▼ Select location     ] [+ New]               │
│  Promise Date [📅 _______________] (optional)                │
│  Quote        [$_________________] (optional)                │
│  Rush         [ ] Mark as rush                               │
│                                                              │
│  ── Photos ────────────────────────────────────────────────  │
│  [Drop files here or click to upload]                        │
│  Minimum 1 photo required                                    │
│                                                              │
│  [img] [img] [img]                                           │
│                                                              │
│  ─────────────────────────────────────────────────────────   │
│                                                              │
│                              [Cancel]  [Create & Print]      │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Flow:**
1. Staff fills form (customer search auto-completes if match found)
2. Staff uploads at least 1 photo
3. Staff clicks "Create & Print"
4. Employee ID prompt appears
5. Ticket created, PDF opens in new tab
6. Staff prints receipt + tag
7. Modal closes, ticket appears in Intake lane

**Customer search:**
- As user types name, dropdown shows matching existing customers
- Selecting auto-fills phone/email
- If no match, new customer created on submit

**Storage location:**
- Dropdown of active locations
- Admin sees "+ New" option to add location inline

---

### 4. Employee ID Entry (Modal)

Simple modal that appears before any attributed action.

```
┌─────────────────────────────────────────┐
│                                         │
│         Enter Employee ID               │
│                                         │
│         [____________________]          │
│                                         │
│              [Confirm]                  │
│                                         │
└─────────────────────────────────────────┘
```

**Behavior:**
- Focus auto-set to input field
- Enter key submits
- Validates against employee list
- Shows error if invalid, clears field
- On success, proceeds with action
- No "remember me" - always prompts for each action

---

### 5. Search Results

Accessed via header search bar.

```
┌──────────────────────────────────────────────────────────────┐
│  Search: "jane"                                    [X]       │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  3 results                                                   │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ JR-0042  •  Jane Doe  •  Gold band  •  In Progress      │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ JR-0038  •  Jane Doe  •  Pearl necklace  •  Closed      │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ JR-0015  •  Jane Smith  •  Watch band  •  Archived      │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  Filters: [Status ▼] [Date range ▼]                          │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

- Click result opens ticket detail
- Closed/Archived tickets searchable
- Status badge indicates current state

---

### 6. Admin Settings

Accessed via settings gear in header. Requires admin PIN.

```
┌──────────────────────────────────────────────────────────────┐
│  Settings                                       [X] Close    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  [Store Info] [Employees] [Locations] [Appearance]           │
│                                                              │
│  ── Store Information ─────────────────────────────────────  │
│  Store Name    [Example Jewelers_______]                     │
│  Phone         [555-0000_______________]                     │
│  Address       [123 Main St____________]                     │
│  Ticket Prefix [JR]                                          │
│                                                              │
│                                    [Save Changes]            │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Tabs:**
- **Store Info:** Name, contact, ticket prefix
- **Employees:** List, add, edit, deactivate employees
- **Locations:** Manage storage locations
- **Appearance:** Theme switcher (Imperial / Retro)

---

## Component Library

Using **Tailwind CSS** for styling with custom theme configuration.

Component approach:
- Build simple, reusable components (Button, Input, Card, Modal, Badge)
- No external component library initially - keeps bundle small and styling consistent
- Can evaluate shadcn-svelte or similar later if needed

---

## Responsive Considerations

Desktop-first, but should remain usable on tablet:
- Minimum supported width: 1024px
- Workboard lanes may stack or scroll horizontally on smaller screens
- Modals should not exceed viewport

---

## Accessibility

- Keyboard navigation for all interactive elements
- Focus indicators visible
- Sufficient color contrast (WCAG AA minimum)
- Form labels properly associated
- Error messages announced to screen readers

---

## Print Styles

Receipt and tag PDFs generated server-side:
- Receipt: Letter size (8.5x11), portrait
- Tag: Small label format (2x1 inch assumed, configurable later)
- Both include ticket code prominently
- QR code optional (links to ticket detail)
