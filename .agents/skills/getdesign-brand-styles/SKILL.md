---
name: getdesign-brand-styles
description: Apply frontend styling guidance distilled from getdesign.md DESIGN.md references, especially when the user wants a Tesla-inspired showroom aesthetic or a Cal.com-inspired neutral SaaS aesthetic. Use for UI design, component styling, landing pages, admin/product surfaces, or when choosing between photography-first minimalism and clean scheduling-tool simplicity.
---

# GetDesign Brand Styles

Use this skill when UI work should follow one of two specific public design languages:

- `Tesla`: photography-first showroom minimalism, radical subtraction, edge-to-edge hero imagery, almost no decoration
- `Cal.com`: neutral open-source SaaS clarity, white canvas, black CTAs, rounded cards, product-UI-in-card presentation

## Choose the right reference

- Use `references/tesla.md` when the interface should feel premium, sparse, product-hero-driven, cinematic, and visually quiet.
- Use `references/cal.md` when the interface should feel practical, neutral, scheduling/productivity-oriented, and easy to extend into dashboards or SaaS flows.
- If the user asks for “clean SaaS”, “scheduling”, “developer tool”, or “open-source product UI”, prefer `cal.md`.
- If the user asks for “luxury product”, “showroom”, “automotive”, “full-screen hero”, or “minimal product marketing”, prefer `tesla.md`.

## Working rules

- Read only the reference that matches the chosen direction unless the task explicitly asks for comparison or blending.
- Preserve the reference’s core tension instead of copying isolated tokens.
- For Tesla-inspired work:
  Focus on subtraction, photography dominance, flat surfaces, restrained typography, and sparse CTA count.
- For Cal-inspired work:
  Focus on neutral surfaces, soft card rounding, strong content structure, black primary actions, and product-like UI fragments.

## Output expectations

- Translate the chosen reference into concrete implementation choices:
  colors, typography, spacing, radius, CTA treatment, shell structure, and component behavior.
- Keep naming explicit when handing work to code:
  cite the chosen style direction and the core traits you are applying.
- If the user asks for a new page or component, adapt the reference to the product context instead of cloning the original brand literally.
