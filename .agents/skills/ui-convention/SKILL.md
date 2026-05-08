---
name: ui-convention
description: Apply the repository's cinematic, hyper-symmetrical UI convention when designing premium frontend experiences, especially for brand sites, galleries, luxury commerce, canvas editors, and editorial interfaces that need motion-led presentation instead of generic dashboard layouts.
---

Use this skill when the user asks for UI work that should follow the project's house visual language rather than ad hoc styling. This skill defines a specific direction: hyper-symmetrical composition, motion-led storytelling, cinematic color grading, strong type contrast, and mechanical interaction feedback.

## Core Rule

Default to a centered visual axis. Functional elements should feel mirrored, balanced, or intentionally counterweighted around a stable centerline. Do not fall back to generic F-pattern dashboards unless the product workflow strictly requires it.

## Layout Convention

- Use `center-aligned` composition as the primary organizing rule.
- Build sections around a strong vertical axis with mirrored or balanced component placement.
- Prefer bilateral balance over casual asymmetry.
- Treat each major module as a self-contained `diorama`: dense, staged, and visually complete.
- For scrolling pages, think in terms of `camera movement`, not stacked cards.
- Preserve a stable horizon. Avoid tilted compositions, chaotic parallax, or floating-card drift.

## Motion Convention

- Replace static hero images with short-loop motion whenever practical.
- Motion should behave like `tracking shots`: calm lateral or forward movement, not random bouncing.
- Use `architectural wipes`: transitions should be masked by real layout edges such as dividers, panels, frames, or viewport cuts.
- Favor quick, deliberate push-ins and pull-outs over soft default easing.
- For scroll progression, prefer stepped scene changes or section snapping when it supports the narrative.

## Color Convention

- Think in terms of `cinematic color grading`, not isolated palette picks.
- Prefer high-lightness, controlled-saturation palettes with warm highlights and creamy shadow transitions.
- Use section-level color coding to imply movement through spaces:
  - showcase zones can lean blush / rose
  - service or systems zones can lean mint / sage
  - archival or editorial zones can lean parchment / smoke
- Avoid loud neon gradients and default purple AI-product color schemes.

## Typography Convention

- Pair a `geometric sans` for headings with a `monospace / typewriter` face for body or annotation.
- Headings should feel ordered, authoritative, and spatially precise.
- Supporting text should feel documentary, editorial, or archival rather than app-generic.
- Maintain strong contrast between display and body styles.
- Avoid default stacks such as `Inter`, `Arial`, or platform system UI as the primary visual voice.

## Interaction Convention

- Favor `mechanical feedback` over soft, vague motion.
- Buttons should feel decisive: click states should read as a firm step, notch, or latch.
- If audio is allowed, short physical click sounds are appropriate.
- Progress indicators should resemble devices or instruments, such as floor indicators, track counters, or carriage markers.
- Horizontal swipes and discrete step transitions are preferred over mushy continuous interpolation.

## Implementation Rules

- When building premium marketing, gallery, or luxury interfaces, prioritize symmetry first, then motion, then copy density.
- When building canvas editors or low-code tools, keep the shell symmetrical and cinematic, but let the working surface stay utilitarian.
- Use real layout edges, frames, separators, gutters, and masks to carry transitions.
- Use video or motion backgrounds only when they are stable, loop cleanly, and do not fight text legibility.
- Keep interactivity crisp. Avoid excessive spring physics.

## Avoid

- Generic SaaS dashboard cards floating on a flat background
- Random asymmetry without a visual axis
- Purple-on-white AI branding defaults
- Soft glassmorphism everywhere
- Hero sections that are just text next to an image block
- Overly smooth, inertial motion that removes all mechanical certainty

## Fast Checklist

- Is there a visible center axis?
- Do sections feel staged like miniature sets?
- Does motion replace at least some static imagery?
- Is the palette graded by section instead of globally flat?
- Are heading and body fonts strongly contrasted?
- Do clicks and transitions feel deliberate and mechanical?
