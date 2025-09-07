# Documentation Deployment Setup

This guide explains how to set up the automated documentation deployment from this repository to `open-prophetdb/biomedgps-docs`.

## Current Issue

The GitHub Action is failing with:
```
Error: Input required and not supplied: token
```

This happens because the `DOCS_DEPLOY_TOKEN` secret is not configured.

## Setup Instructions

### Step 1: Create Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Configure the token:
   - **Name**: `BioMedGPS Docs Deployment`
   - **Expiration**: Choose appropriate duration (recommend 90 days or 1 year)
   - **Scopes**: Select these permissions:
     - ✅ `repo` (Full control of private repositories)
     - ✅ `workflow` (Update GitHub Action workflows)

4. Click "Generate token"
5. **Important**: Copy the token value immediately (you won't see it again)

### Step 2: Add Repository Secret

1. Go to your BioMedGPS repository on GitHub
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Configure the secret:
   - **Name**: `DOCS_DEPLOY_TOKEN`
   - **Secret**: Paste the token value from Step 1
5. Click "Add secret"

### Step 3: Verify Target Repository

Ensure the target repository `open-prophetdb/biomedgps-docs` exists and:
- Has GitHub Pages enabled (Settings → Pages → Source: Deploy from a branch)
- The token has write access to this repository
- You are a member of the `open-prophetdb` organization with appropriate permissions

### Step 4: Test Deployment

After setting up the token, you can test the deployment by:

1. **Manual Trigger**: Go to Actions → Deploy Documentation → Run workflow
2. **Push Changes**: Make a small change to any file in the `docs/` directory and push to `main` or `dev`

## Alternative: GitHub App Token

For better security and organization-wide access, consider using a GitHub App instead of a personal access token:

1. Create a GitHub App for the organization
2. Install it on both repositories
3. Generate an installation access token
4. Use the app's token instead of a personal token

## Troubleshooting

### Token Permission Issues
If deployment fails with permission errors:
- Verify the token has `repo` scope
- Check that you have write access to `open-prophetdb/biomedgps-docs`
- Ensure you're a member of the `open-prophetdb` organization

### Target Repository Issues
If the target repository doesn't exist:
```bash
# Create it manually or ask organization admin
# Repository should be public for GitHub Pages
```

### Workflow Fails After Token Setup
1. Check the Actions log for specific error messages
2. Verify the secret name is exactly `DOCS_DEPLOY_TOKEN`
3. Ensure the token hasn't expired
4. Check repository permissions

## Security Notes

- **Never commit tokens to the repository**
- Use repository secrets, not hardcoded values
- Consider token expiration and renewal
- Use minimal required permissions
- Consider using GitHub Apps for organization-wide deployments

## Manual Deployment (Fallback)

If automated deployment isn't working, you can manually deploy:

```bash
# Build documentation locally
cd docs
bundle install
bundle exec jekyll build

# Clone target repository
git clone https://github.com/open-prophetdb/biomedgps-docs.git
cd biomedgps-docs

# Copy files
mkdir -p biomedgps
cp -r ../docs/_site/* biomedgps/

# Commit and push
git add biomedgps/
git commit -m "Update BioMedGPS documentation"
git push origin main
```

## Contact

If you continue to have issues:
- Check the [GitHub Actions documentation](https://docs.github.com/en/actions)
- Create an issue in this repository
- Contact the repository maintainers

---

**Next Steps**: Once you've set up the `DOCS_DEPLOY_TOKEN` secret, the workflow should run successfully and deploy documentation to `https://open-prophetdb.github.io/biomedgps-docs/biomedgps/`