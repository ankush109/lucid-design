Use these free CDN URLs — no API keys required. Choose dimensions that match
the layout slot (hero, card, avatar). Always include descriptive alt text.

1. Topical photos (keyword-tagged, Creative Commons):
   https://loremflickr.com/{WIDTH}/{HEIGHT}/{KEYWORD1},{KEYWORD2}?lock={SEED}
   Example (studio photography hero):
   <img src="https://loremflickr.com/1600/900/studio,photography,camera?lock=42"
        alt="Photographer setting up lights in a studio" />

2. Deterministic random photos (topic-less but reliable):
   https://picsum.photos/seed/{DESCRIPTIVE-SEED}/{WIDTH}/{HEIGHT}
   Example: https://picsum.photos/seed/pricing-hero/1200/600

3. User avatars (1-70, real headshot-style):
   https://i.pravatar.cc/{SIZE}?img={1..70}
   Example: https://i.pravatar.cc/72?img=13

Image placement principles:
- One large hero image, aspect ratio 16:9 or 3:2. Match keyword to the subject.
- Feature cards benefit from a moment/product photo, not just an icon.
- Testimonials use real-looking avatars via pravatar (pick different img= numbers).
- For text over images add a scrim / gradient overlay (contrast floor 4.5:1).
- Consistent aspect ratios across peer elements (all feature cards use 4:3, etc.).
- Loading: reserve the image's aspect ratio in CSS so nothing shifts.
- Never use lorem ipsum captions — write realistic ones tied to the subject.
