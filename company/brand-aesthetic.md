# GUAC Brand Aesthetic System

## Design Philosophy

GUAC's visual identity is built on a single idea: **memory should feel structured, alive, and trustworthy.**

The aesthetic is:
- **Organic geometry** — rounded shapes inspired by the avocado pit and branching neurons.
- **Deep green authority** — dark forest greens signal stability and growth.
- **Lime vitality** — bright accents feel electric and alive, like cognition firing.
- **Monospace precision** — headings in JetBrains Mono communicate technical correctness.
- **Generous whitespace** — every element breathes; density is intentional, never accidental.

---

## Color System

### Light Mode

| Token | Hex | Usage |
|-------|-----|-------|
| `--guac-core` | `#0D3B2E` | Primary brand, hero backgrounds, headings. |
| `--guac-pit` | `#7C9A27` | Secondary accent, links, icons. |
| `--guac-flesh` | `#C8F560` | CTAs, highlights, success states, code strings. |
| `--ink` | `#0D1117` | Body text, dark surfaces. |
| `--stone` | `#24292F` | Secondary text, muted copy. |
| `--mist` | `#F6F8FA` | Page backgrounds. |
| `--cloud` | `#FFFFFF` | Cards, elevated surfaces. |
| `--line` | `#D0D7DE` | Borders, dividers. |

### Dark Mode

| Token | Hex | Usage |
|-------|-----|-------|
| `--guac-core` | `#0A2E24` | Dark hero surfaces. |
| `--guac-pit` | `#8FAB3A` | Dark-mode accent (slightly brighter for contrast). |
| `--guac-flesh` | `#D4FF6E` | Dark-mode CTA and highlights. |
| `--ink` | `#F6F8FA` | Dark-mode body text. |
| `--stone` | `#9FA8B1` | Dark-mode secondary text. |
| `--mist` | `#0D1117` | Dark-mode page background. |
| `--cloud` | `#161B22` | Dark-mode cards. |
| `--line` | `#30363D` | Dark-mode borders. |

### Usage Rules

- Use `--guac-core` for the most important surfaces and the logo.
- Use `--guac-flesh` sparingly; it should feel like an electric spark, not wallpaper.
- Keep text on `--guac-core` in `--cloud` or `--guac-flesh` for accessibility.
- Never use `--guac-flesh` for body text.

---

## Typography

| Role | Font | Weight | Use |
|------|------|--------|-----|
| Headings | JetBrains Mono | 700 | H1–H3, logo, labels. |
| Body | Inter | 400/500/600 | Paragraphs, UI copy, buttons. |
| Code | JetBrains Mono | 400 | Inline code, blocks, diagrams. |

### Type Scale

| Token | Desktop | Mobile | Line |
|-------|---------|--------|------|
| `text-hero` | 4rem | 2.5rem | 1.05 |
| `text-h1` | 2.5rem | 2rem | 1.1 |
| `text-h2` | 1.75rem | 1.5rem | 1.2 |
| `text-h3` | 1.25rem | 1.125rem | 1.3 |
| `text-body` | 1rem | 1rem | 1.6 |
| `text-small` | 0.875rem | 0.875rem | 1.5 |
| `text-mono` | 0.875rem | 0.8125rem | 1.5 |

### Typography Rules

- Headings are sentence case unless they are the brand name.
- Body text never exceeds 70 characters per line; use `max-width: 70ch` for prose.
- Code blocks use `font-variant-ligatures: none` to avoid confusing ligatures.

---

## Spacing & Grid

### Base Unit

The base unit is **0.25rem (4px)**. All spacing tokens are multiples of this unit.

| Token | Value |
|-------|-------|
| `--space-xs` | 0.25rem |
| `--space-sm` | 0.5rem |
| `--space-md` | 1rem |
| `--space-lg` | 1.5rem |
| `--space-xl` | 2.5rem |
| `--space-2xl` | 4rem |
| `--space-3xl` | 6rem |

### Grid

- Max content width: `--max-width: 72rem`.
- Prose width: `--content-width: 48rem`.
- Page padding: `--space-xl` on desktop, `--space-md` on mobile.
- Cards and grids use CSS Grid with `minmax()` for responsiveness.

---

## Elevation & Shape

### Radii

| Token | Value | Use |
|-------|-------|-----|
| `--radius-sm` | 0.375rem | Buttons, badges, inputs. |
| `--radius-md` | 0.75rem | Cards, code blocks. |
| `--radius-lg` | 1rem | Large cards, sections. |
| `--radius-full` | 9999px | Pills, avatars. |

### Shadows

| Token | Light Mode | Dark Mode |
|-------|------------|-----------|
| `--shadow-sm` | `0 1px 2px rgba(13,17,23,0.08)` | `0 1px 2px rgba(0,0,0,0.25)` |
| `--shadow-md` | `0 4px 12px rgba(13,17,23,0.1)` | `0 4px 12px rgba(0,0,0,0.35)` |
| `--shadow-lg` | `0 12px 32px rgba(13,17,23,0.14)` | `0 12px 32px rgba(0,0,0,0.45)` |

---

## Motion & Animation

### Principles

- Motion should feel **responsive**, not decorative.
- Use short durations: **150–300ms**.
- Prefer `transform` and `opacity` for performance.
- Respect `prefers-reduced-motion`.

### Standard Transitions

| Use | Duration | Easing |
|-----|----------|--------|
| Hover lift | 150ms | `ease` |
| Colour change | 200ms | `ease` |
| Section reveal | 300ms | `cubic-bezier(0.16, 1, 0.3, 1)` |
| Code block fade | 200ms | `ease-in-out` |

### Scroll Behaviour

- Smooth scroll for anchor links.
- Section fade-in on scroll using Intersection Observer.
- Sticky nav with backdrop blur.

---

## Iconography

- Use **Lucide-style** icons: 1.5px stroke, rounded caps, 24×24 default.
- Icons should be functional, not ornamental.
- Colour icons only with `--guac-pit` or `--guac-flesh` on dark surfaces.

---

## Patterns & Textures

### Dot Grid

A subtle dot grid using `--guac-pit` at 10% opacity. Used in hero and architecture sections.

```css
background-image: radial-gradient(var(--guac-pit) 1px, transparent 1px);
background-size: 24px 24px;
opacity: 0.08;
```

### Branch Motif

Abstract branching lines inspired by Git graphs and neurons. Used as section dividers or footer decoration.

---

## Logo System

### Variants

- `logo-dark.svg` — full colour on light backgrounds.
- `logo-light.svg` — light version on dark backgrounds.
- `logo-icon.svg` — mark only, for favicons and avatars.
- `wordmark.svg` — text only, for narrow spaces.

### Clear Space

Always maintain clear space around the logo equal to the height of the letter "U" in the wordmark.

### Minimum Size

- Wordmark: 80px wide.
- Icon: 16px.

### Don'ts

- Don't stretch or rotate the logo.
- Don't add drop shadows or glows.
- Don't change the colours outside the approved palettes.
- Don't use the icon smaller than 16px.

---

## Photography & Imagery

### Style

- Macro photography of natural textures: leaves, stone, circuit boards.
- Dark, moody lighting with green-tinted highlights.
- Avoid generic AI-generated stock humans.

### Treatment

- Desaturate slightly, then lift greens and teals.
- Add subtle film grain for tactility.
- Use duotone overlays sparingly with `--guac-core` and `--guac-flesh`.

---

## Dark Mode

All GUAC web properties must support dark mode.

- Default to system preference via `prefers-color-scheme`.
- Provide a manual toggle that persists in `localStorage`.
- Toggle should be a sun/moon icon in the nav.
- Never hard-code colours; always use CSS custom properties.

---

## Accessibility

- Minimum contrast ratio: 4.5:1 for body text, 3:1 for large text.
- Focus rings: 2px solid `--guac-flesh` with 2px offset.
- Interactive elements must have visible focus states.
- Use semantic HTML and ARIA labels where native semantics are insufficient.
