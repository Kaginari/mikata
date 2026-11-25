import DefaultTheme from 'vitepress/theme';
import '@mikata/theme/tokens.css';
import './style.css';
import Landing from './Landing.vue';

// VitePress marks dark mode with `html.dark`; the tokens read `data-theme`. Mirror the one onto
// the other, so the appearance toggle switches the whole theme.
export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    // `layout: landing` in a page's frontmatter draws the front page.
    app.component('landing', Landing);
    if (typeof document === 'undefined') return;
    const root = document.documentElement;
    const sync = () =>
      root.setAttribute('data-theme', root.classList.contains('dark') ? 'dark' : 'light');
    sync();
    new MutationObserver(sync).observe(root, { attributes: true, attributeFilter: ['class'] });
  },
};
