// Kit catalog — mirrors the backend themes.md / variants.md.
export const KIT_THEMES = [
  { id: 'auto', label: 'Auto — pick for me' },
  { id: 'editorial-warm-cream', label: 'Editorial · warm cream' },
  { id: 'editorial-dark-refined', label: 'Editorial · dark refined' },
  { id: 'saas-minimal-white', label: 'SaaS · minimal white' },
  { id: 'saas-corporate-blue', label: 'SaaS · corporate blue' },
  { id: 'playful-vibrant', label: 'Playful vibrant' },
  { id: 'technical-terminal', label: 'Technical · terminal' },
  { id: 'luxury-fashion', label: 'Luxury · fashion' },
  { id: 'warm-hospitality', label: 'Warm · hospitality' },
  { id: 'clinical-healthcare', label: 'Clinical · healthcare' },
  { id: 'brutalist-mono', label: 'Brutalist mono' },
  { id: 'cyber-neon-dark', label: 'Cyber · neon dark' },
  { id: 'forest-organic', label: 'Forest · organic' },
];

export const KIT_PALETTES = [
  { id: 'auto', label: 'Match theme' },
  { id: 'warm-cream-brick', label: 'Warm cream + brick' },
  { id: 'minimal-white', label: 'Minimal white' },
  { id: 'dark-refined', label: 'Dark refined' },
  { id: 'corporate-blue', label: 'Corporate blue' },
  { id: 'warm-earth', label: 'Warm earth' },
  { id: 'clinical-teal', label: 'Clinical teal' },
  { id: 'fashion-mono', label: 'Fashion mono' },
  { id: 'cyber-neon', label: 'Cyber neon' },
  { id: 'sunset-terracotta', label: 'Sunset terracotta' },
  { id: 'forest-paper', label: 'Forest paper' },
];

export const KIT_VARIANTS = {
  navbar: [
    { id: 'auto', label: 'Auto' },
    { id: 'nav-01', label: '01 · Brand-heavy serif' },
    { id: 'nav-02', label: '02 · Sticky transparent' },
    { id: 'nav-03', label: '03 · Centered editorial' },
  ],
  hero: [
    { id: 'auto', label: 'Auto' },
    { id: 'hero-01', label: '01 · Centered editorial' },
    { id: 'hero-02', label: '02 · Split product shot' },
    { id: 'hero-03', label: '03 · Bento with stats' },
  ],
  features: [
    { id: 'auto', label: 'Auto' },
    { id: 'features-01', label: '01 · Alternating rows' },
    { id: 'features-02', label: '02 · Bento varied' },
    { id: 'features-03', label: '03 · Icon triad' },
  ],
  testimonials: [
    { id: 'auto', label: 'Auto' },
    { id: 'testimonials-01', label: '01 · Three-card grid' },
    { id: 'testimonials-02', label: '02 · Hero quote' },
  ],
  pricing: [
    { id: 'auto', label: 'Auto' },
    { id: 'pricing-01', label: '01 · Three-tier' },
    { id: 'pricing-02', label: '02 · Single plan' },
  ],
  cta: [
    { id: 'auto', label: 'Auto' },
    { id: 'cta-01', label: '01 · Centered band' },
    { id: 'cta-02', label: '02 · Split with form' },
  ],
  footer: [
    { id: 'auto', label: 'Auto' },
    { id: 'footer-01', label: '01 · Minimal hairline' },
    { id: 'footer-02', label: '02 · Giant wordmark' },
  ],
};

// Landing-page regex — verbatim from ui.html send().
export const LANDING_PAGE_PATTERN =
  /\b(landing page|landing|marketing site|marketing page|homepage|home page|saas site|product page|launch page|hero page|sales page|splash page|coming soon|waitlist page)\b/i;
