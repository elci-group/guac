# GUAC Site Kit

A lightweight, dependency-free design system for GUAC web properties.

## Files

- `tokens.css` — CSS custom properties for color, typography, spacing, radii, and shadows.
- `components.html` — A living reference page for nav, hero, cards, code blocks, and footer.

## Usage

1. Copy `tokens.css` into your project.
2. Load it before your own stylesheet:
   ```html
   <link rel="stylesheet" href="site-kit/tokens.css">
   <link rel="stylesheet" href="styles.css">
   ```
3. Use the custom properties in your CSS:
   ```css
   .hero {
     background: var(--guac-core);
     color: var(--cloud);
     padding: var(--space-3xl) var(--space-xl);
     border-radius: var(--radius-lg);
   }
   ```

## Fonts

The kit uses **Inter** for body text and **JetBrains Mono** for headings and code. Load them from Google Fonts or self-host:

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
```

## Principles

- **Dark green core, lime accents.** The brand is technical but alive.
- **Monospace headings, sans-serif body.** Precision plus readability.
- **Generous spacing.** Let the architecture breathe.
- **Sharp radii, soft shadows.** Modern without being clinical.

## Extending

When adding new components, prefer CSS custom properties over hard-coded values. Document new tokens here if they are reusable.
