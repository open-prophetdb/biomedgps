# BioMedGPS Documentation

This directory contains the Jekyll-based documentation for BioMedGPS. The documentation is automatically deployed to the [open-prophetdb/biomedgps-docs](https://github.com/open-prophetdb/biomedgps-docs) repository.

## Documentation Structure

```
docs/
├── _config.yml           # Jekyll configuration
├── Gemfile              # Ruby dependencies
├── index.md             # Homepage
├── getting-started.md   # Installation and setup guide
├── user-guide.md        # Comprehensive user guide
├── features.md          # Features overview
├── api-reference.md     # API documentation
├── faq.md              # Frequently asked questions
├── _guides/            # Detailed guides collection
│   └── knowledge-graph-exploration.md
└── _features/          # Feature-specific documentation
```

## Local Development

### Prerequisites
- Ruby 3.0+
- Bundler

### Setup
```bash
cd docs
bundle install
```

### Build and Serve
```bash
# Serve locally with auto-reload (uses development config)
bundle exec jekyll serve --config _config.yml,_config-dev.yml

# Build static site for production (with correct baseurl)
JEKYLL_ENV=production bundle exec jekyll build --baseurl "/biomedgps"

# Build for local testing
bundle exec jekyll build
```

The site will be available at `http://localhost:4000`.

**Important**: When developing locally, use the development config to avoid subdirectory path issues. The production build automatically handles the correct paths for GitHub Pages deployment.

## Deployment

Documentation is automatically deployed via GitHub Actions when:
- Changes are pushed to `main` or `dev` branches in the `docs/` directory
- Pull requests modify documentation files
- Manual workflow dispatch

### Deployment Process
1. **Build**: Jekyll builds the static site
2. **Deploy**: Files are copied to `open-prophetdb/biomedgps-docs/biomedgps/`
3. **Validate**: Checks that deployed pages are accessible

### Live Documentation
The deployed documentation is available at:
https://open-prophetdb.github.io/biomedgps-docs/biomedgps/

## Configuration Requirements

### GitHub Secrets
The deployment workflow requires a GitHub token with write access to the target repository:

```
DOCS_DEPLOY_TOKEN: A personal access token or GitHub App token with write access to open-prophetdb/biomedgps-docs
```

### Repository Permissions
The token must have:
- `contents:write` permission on `open-prophetdb/biomedgps-docs`
- Access to public repositories (if using a personal access token)

## Content Guidelines

### Writing Style
- Use clear, concise language
- Include practical examples
- Provide step-by-step instructions
- Add screenshots where helpful

### Structure
- Start with overview/introduction
- Progress from basic to advanced topics
- Include troubleshooting sections
- Cross-reference related content

### Markdown Features
The documentation supports:
- Standard Markdown syntax
- Jekyll front matter
- Code syntax highlighting
- Tables and lists
- Custom CSS styling

## Contributing

### Adding New Pages
1. Create new `.md` files in the `docs/` directory
2. Add appropriate front matter:
   ```yaml
   ---
   layout: page
   title: Page Title
   permalink: /page-url/
   ---
   ```
3. Update navigation in `_config.yml` if needed

### Adding to Collections
- **Guides**: Add to `_guides/` directory
- **Features**: Add to `_features/` directory

### Images and Assets
- Store in `docs/assets/` directory
- Reference as `/assets/image.png` in markdown
- Optimize images for web (compress, appropriate size)

### Testing Changes
1. Test locally with `bundle exec jekyll serve`
2. Check all links work correctly
3. Verify responsive design on mobile
4. Validate HTML and accessibility

## Troubleshooting

### Common Issues

**Bundle install fails**
```bash
# Update bundler
gem update bundler
bundle install
```

**Jekyll build errors**
- Check for YAML front matter syntax
- Verify all links and references
- Check for unsupported plugins

**Deployment fails**
- Verify `DOCS_DEPLOY_TOKEN` is set correctly
- Check repository permissions
- Review workflow logs in GitHub Actions

### Getting Help
- Check existing GitHub issues
- Review Jekyll documentation
- Test changes locally before pushing

## Maintenance

### Regular Tasks
- Update Ruby gems: `bundle update`
- Check for broken links
- Review and update content accuracy
- Monitor deployment workflow status

### Content Updates
Documentation should be updated when:
- New features are added to BioMedGPS
- API endpoints change
- Installation procedures change
- User feedback indicates confusion or gaps

The documentation is a living resource that should grow and improve alongside the BioMedGPS platform.