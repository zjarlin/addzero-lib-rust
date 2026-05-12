# Tesla Reference

Source basis:
- [getdesign.md Tesla page](https://getdesign.md/tesla/design-md)
- installed `DESIGN.md` via `npx getdesign@latest add tesla`

## Use this reference for

- automotive, luxury hardware, or flagship product marketing
- landing pages where one hero image should carry most of the emotional weight
- UI that should feel engineered, sparse, and extremely controlled

## Core design read

Tesla is not a general component-rich system. It is a product showroom with near-zero chrome.
The strongest pattern is radical subtraction: imagery is primary, UI is secondary, decoration is almost absent.

## Visual rules

- Use full-viewport sections for major beats.
- Let one product, one headline, and at most one CTA pair own each screen.
- Avoid shadows, gradients, ornamental borders, background patterns, and decorative flourishes.
- Prefer white, dark graphite, and one electric-blue CTA color.
- Keep surfaces flat. Layering comes from imagery and z-index, not elevation effects.

## Palette

- Primary CTA blue: `#3E6AE1`
- Main dark text: `#171A20`
- Body text: `#393C41`
- Secondary text: `#5C5E62`
- Surface white: `#FFFFFF`
- Alternate light surface: `#F4F4F4`
- Dividers/hairlines: `#EEEEEE` and `#D0D1D2`

## Typography

- Use a geometric, engineered sans.
- Keep weights narrow: mostly `400` and `500`.
- Hero titles can be large, but avoid flamboyant display tricks.
- Avoid uppercase-heavy treatment unless the surrounding brand truly requires it.
- Keep letter spacing mostly normal.

## Components

- Buttons:
  4px radius, sharp technical feel, blue primary / white secondary.
- Navigation:
  floating over hero, minimal framing, simple text buttons.
- Cards:
  usually borderless and shadowless. If cards exist, they should feel like content containers, not visual objects.

## Layout

- Think in staged screens, not dense dashboards.
- Preserve large white or image-led negative space.
- Keep CTA count low.
- Use horizontal centering and simple vertical rhythm under the hero.

## Avoid

- dashboard-card clutter
- colorful secondary accents
- soft glassmorphism
- animated bounce or playful micro-interactions
- multiple competing panels above the fold

## Translate into implementation

When coding:

- prefer one dominant background image or product render
- use flat CSS variables with almost no shadow tokens
- keep interactive motion subtle and short
- restrict accent color use to true primary actions
- remove unnecessary helper text, chips, badges, and separators
