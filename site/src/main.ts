// COD Site — TypeScript entry point

interface BenchFill extends HTMLElement {
  _targetWidth?: string;
}

const observe = (el: Element, clazz: string, threshold = 0.1): void => {
  const obs = new IntersectionObserver((entries) => {
    entries.forEach((e) => {
      if (e.isIntersecting) {
        e.target.classList.add(clazz);
        obs.unobserve(e.target);
      }
    });
  }, { threshold, rootMargin: '0px 0px -40px 0px' });
  obs.observe(el);
};

const staggerIn = (el: HTMLElement, delay: number): void => {
  el.style.opacity = '0';
  el.style.transform = 'translateY(20px)';
  el.style.transition = `opacity 0.6s cubic-bezier(0.22,1,0.36,1) ${delay}s, transform 0.6s cubic-bezier(0.22,1,0.36,1) ${delay}s`;
  requestAnimationFrame(() => {
    el.style.opacity = '1';
    el.style.transform = 'translateY(0)';
  });
};

document.addEventListener('DOMContentLoaded', () => {

  // ---- Scroll reveal for sections ----
  document.querySelectorAll(
    '.bench-card, .module-card, .removed-card, .hero-content, ' +
    '.section-title, .section-sub, .section-label, .cta-section, ' +
    '.retained-note, .hero-stats'
  ).forEach((el) => observe(el, 'visible'));

  // ---- Animate benchmark bars on scroll ----
  document.querySelectorAll<BenchFill>('.bench-fill').forEach((bar) => {
    const target = bar.style.width;
    bar.style.width = '0%';
    const obs = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          setTimeout(() => { bar.style.width = target; }, 200);
          obs.unobserve(bar);
        }
      });
    }, { threshold: 0.3 });
    obs.observe(bar);
  });

  // ---- Hero parallax glow ----
  const hero = document.querySelector<HTMLElement>('.hero');
  const glow = document.querySelector<HTMLElement>('.hero-glow');
  if (hero && glow) {
    hero.addEventListener('mousemove', ((e: MouseEvent) => {
      const rect = hero.getBoundingClientRect();
      const x = ((e.clientX - rect.left) / rect.width - 0.5) * 20;
      const y = ((e.clientY - rect.top) / rect.height - 0.5) * 20;
      glow.style.transform = `translate(calc(-50% + ${x}px), ${y}px)`;
    }) as EventListener);
  }

  // ---- Smooth nav scroll ----
  document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
    anchor.addEventListener('click', (e) => {
      const href = (anchor as HTMLAnchorElement).getAttribute('href');
      if (!href || href === '#') return;
      e.preventDefault();
      const target = document.querySelector(href);
      if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    });
  });

  // ---- Staggered hero title ----
  document.querySelectorAll('.hero-title-line').forEach((line, i) => {
    staggerIn(line as HTMLElement, 0.4 + i * 0.15);
  });

  // ---- Badge fade-in ----
  const badge = document.querySelector<HTMLElement>('.hero-badge');
  if (badge) staggerIn(badge, 0.2);

  // ---- Hero sub + actions ----
  const heroSub = document.querySelector<HTMLElement>('.hero-sub');
  const heroActions = document.querySelector<HTMLElement>('.hero-actions');
  if (heroSub) staggerIn(heroSub, 0.9);
  if (heroActions) staggerIn(heroActions, 1.05);

  // ---- Stats stagger ----
  document.querySelectorAll<HTMLElement>('.stat').forEach((stat, i) => {
    staggerIn(stat, 1.4 + i * 0.12);
  });

  // ---- Nav background on scroll ----
  const nav = document.querySelector<HTMLElement>('.nav');
  if (nav) {
    window.addEventListener('scroll', () => {
      nav.style.background = window.scrollY > 80
        ? 'rgba(7,7,13,0.95)'
        : 'rgba(7,7,13,0.8)';
      nav.style.borderBottomColor = window.scrollY > 80
        ? 'rgba(0,212,255,0.1)'
        : 'rgba(255,255,255,0.06)';
    });
  }
});
