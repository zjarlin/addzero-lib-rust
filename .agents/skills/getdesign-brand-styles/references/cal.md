# Cal.com Reference

Source basis:
- [getdesign.md Cal page](https://getdesign.md/cal/design-md)
- installed `DESIGN.md` via `npx getdesign@latest add cal`

## Use this reference for

- scheduling, booking, workspace, productivity, or developer-facing SaaS
- neutral product shells that need to feel modern without looking generic
- interfaces where product UI fragments inside cards should carry the story

## Core design read

Cal.com is a white-canvas SaaS system with black primary actions, clean typography, rounded cards, and strong structural clarity.
It feels practical, open-source, and product-first rather than luxurious or editorial.

## Visual rules

- Use white as the dominant canvas.
- Use black or near-black for the primary CTA and main headings.
- Prefer soft-rounded cards around `12px`.
- Show product surfaces directly inside sections or cards instead of relying on abstract illustration.
- Keep the page calm and neutral; brand energy comes from typography and composition, not loud color.

## Palette

- Primary / ink: `#111111`
- Primary active: `#242424`
- Body text: `#374151`
- Muted text: `#6B7280`
- Hairline: `#E5E7EB`
- Soft surface: `#F8F9FA`
- Card surface: `#F5F5F5`
- Canvas: `#FFFFFF`
- Dark footer / closing band: `#101010`
- Accent blue exists, but should be secondary to the black CTA system: `#3B82F6`

## Typography

- Use a clean geometric display face for headlines when available.
- Pair it with a practical sans for UI and body copy.
- Display headings can be bold and large with slight negative tracking.
- Body copy should remain readable and normal, not editorially stylized.

## Components

- Buttons:
  black primary buttons with white text, medium rounding, compact SaaS sizing.
- Secondary buttons:
  white with dark text, same geometry as primary.
- Cards:
  white or light-gray surfaces with soft radius, generous padding, minimal border noise.
- Pills and tabs:
  quiet surface contrast, rounded, neutral.
- Inputs:
  simple white controls, dark text, medium radius, subtle borders.

## Layout

- Prefer structured sections over full-screen scenes.
- Use clear content bands with strong spacing and product screenshots/mockups.
- Allow denser information than Tesla, but keep it orderly.
- Suitable for marketing pages that transition naturally into application-like surfaces.

## Avoid

- loud gradients
- luxury-showroom emptiness when the product needs functional density
- overly playful color systems
- excessive shadow stacking
- dark mode bias unless the product explicitly needs it

## Translate into implementation

When coding:

- define neutral surface tokens first
- make black primary actions the default emphasis
- use rounded cards and consistent container spacing
- place product screenshots, tables, or calendar/task fragments directly in the layout
- keep footer or terminal sections darker to close the page with contrast
