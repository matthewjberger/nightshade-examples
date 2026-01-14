# Leptos Portfolio Template

A modern portfolio template built with Rust, Leptos, and WebAssembly. Featuring dark mode, smooth animations, and automatic GitHub Pages deployment.

## Features

- 🦀 Built entirely in Rust with Leptos framework
- 🎨 Dark/Light mode with localStorage persistence
- 📱 Responsive design with Tailwind CSS
- ⚡ Fast static site generation with Trunk
- 🚀 Automatic deployment to GitHub Pages
- 🎯 Single-page layout with smooth scrolling
- 📝 Articles section with routing and Leptos component-based articles

## Quick Start

### Development

```bash
just serve
```

### Build

```bash
just build
```

## Customization

1. Update `src/pages/home.rs`:
   - Change "Your Name" to your name
   - Update the job title
   - Update social links (LinkedIn, GitHub)

2. Update `src/pages/about.rs`:
   - Modify the about section text

3. Update `src/pages/experience.rs`:
   - Add your work experience
   - Update the `jobs` vector with your positions
   - Modify job titles, companies, periods, and achievements
   - The section features a toggle between timeline and full view

4. Update `src/pages/projects.rs`:
   - Add your projects with links and descriptions
   - Replace placeholder project data

5. Update `src/pages/contact.rs`:
   - Add your contact information and social links

6. Update `src/components/navigation.rs`:
   - Change the site title in the nav bar

7. Update articles in `src/articles/`:
   - Each article is a separate Leptos component module
   - Add your own articles by creating new modules in `src/articles/`
   - Update `src/articles.rs` to register new articles in `get_all_articles()` and `get_article_component()`
   - Article metadata includes slug, title, date, and description
   - Example articles are provided as templates

## Deployment

### For username.github.io repos

Update `.github/workflows/gh-pages.yml`:
```yaml
- name: Build with Trunk
  run: trunk build --release --public-url /
```

### For project repos (e.g., username.github.io/project-name)

Update `.github/workflows/gh-pages.yml`:
```yaml
- name: Build with Trunk
  run: trunk build --release --public-url /project-name/
```

The site is automatically deployed to GitHub Pages when pushed to the main branch.

## License

MIT
