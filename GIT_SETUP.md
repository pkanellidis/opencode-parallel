# Git Repository Setup

## ✅ Local Repository Initialized

Your git repository has been initialized and the initial commit is complete!

```bash
✓ Git initialized
✓ Branch renamed to 'main'
✓ All files staged
✓ Initial commit created
```

**Commit Details:**
- Commit hash: `62858da`
- Files: 24 files, 4,007 insertions
- Branch: `main`

---

## 🚀 Next Steps: Push to GitHub

### Option 1: Using GitHub CLI (Recommended)

If you have `gh` CLI installed:

```bash
cd ~/opencode-parallel

# Create and push repository in one command
gh repo create opencode-parallel --public --source=. --remote=origin --push

# Or if you want it private
gh repo create opencode-parallel --private --source=. --remote=origin --push
```

### Option 2: Manual GitHub Setup

1. **Create a new repository on GitHub:**
   - Go to https://github.com/new
   - Name: `opencode-parallel`
   - Description: "A CLI tool for running multiple AI coding agents in parallel"
   - Choose Public or Private
   - **Do NOT** initialize with README, .gitignore, or license (we already have these)
   - Click "Create repository"

2. **Add the remote and push:**
   ```bash
   cd ~/opencode-parallel
   
   # Replace YOUR_USERNAME with your GitHub username
   git remote add origin https://github.com/YOUR_USERNAME/opencode-parallel.git
   
   # Push to GitHub
   git push -u origin main
   ```

### Option 3: Using SSH (If you have SSH keys set up)

```bash
cd ~/opencode-parallel

# Replace YOUR_USERNAME with your GitHub username
git remote add origin git@github.com:YOUR_USERNAME/opencode-parallel.git

git push -u origin main
```

---

## 📋 Verify Remote Connection

After adding the remote, verify it:

```bash
git remote -v
```

Should show:
```
origin  https://github.com/YOUR_USERNAME/opencode-parallel.git (fetch)
origin  https://github.com/YOUR_USERNAME/opencode-parallel.git (push)
```

---

## 🔄 Common Git Commands

Once pushed, use these commands for development:

```bash
# Check status
git status

# Add changes
git add .
git add path/to/file

# Commit changes
git commit -m "feat: add new feature"
git commit -m "fix: resolve bug"

# Push changes
git push

# Pull changes
git pull

# View commit history
git log --oneline
git log --graph --oneline --all

# Create a branch
git checkout -b feature/my-feature

# Switch branches
git checkout main
git checkout feature/my-feature

# Merge branch
git checkout main
git merge feature/my-feature

# View diff
git diff
git diff HEAD~1
```

---

## 🏷️ Tagging Releases

When ready to release:

```bash
# Create annotated tag
git tag -a v0.1.0 -m "Initial release"

# Push tag to GitHub
git push origin v0.1.0

# Or push all tags
git push --tags
```

This will trigger the release workflow in `.github/workflows/release.yml`!

---

## 📝 Commit Message Conventions

Follow these conventions for commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting)
- `refactor:` Code refactoring
- `test:` Adding tests
- `chore:` Maintenance tasks
- `perf:` Performance improvements

Examples:
```bash
git commit -m "feat: add real Anthropic API integration"
git commit -m "fix: resolve TUI rendering issue on Windows"
git commit -m "docs: update installation instructions"
git commit -m "refactor: simplify agent state management"
```

---

## 🔒 .gitignore Already Configured

The repository already has a `.gitignore` that excludes:
- `/target` (Rust build artifacts)
- `Cargo.lock` (for libraries)
- IDE files (`.vscode`, `.idea`)
- OS files (`.DS_Store`)
- Config files (`*.local.json`, `.env`)

---

## 🌿 Branching Strategy

Recommended workflow:

```bash
# Main branch (stable)
main

# Development branch
dev

# Feature branches
feature/add-anthropic-api
feature/web-dashboard

# Fix branches
fix/tui-crash
fix/auth-validation

# Release branches
release/v0.1.0
release/v0.2.0
```

Example workflow:
```bash
# Create dev branch
git checkout -b dev

# Create feature branch from dev
git checkout -b feature/new-feature

# Work on feature...
git add .
git commit -m "feat: implement new feature"

# Merge back to dev
git checkout dev
git merge feature/new-feature

# When ready for release, merge dev to main
git checkout main
git merge dev
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin main --tags
```

---

## 🤝 Collaboration

When working with others:

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/opencode-parallel.git
cd opencode-parallel

# Always pull before starting work
git pull

# Create feature branch
git checkout -b feature/my-contribution

# Make changes, commit, push
git push -u origin feature/my-contribution

# Open Pull Request on GitHub
```

---

## 🆘 Common Issues

### "fatal: remote origin already exists"
```bash
git remote remove origin
git remote add origin https://github.com/YOUR_USERNAME/opencode-parallel.git
```

### "Updates were rejected because the remote contains work"
```bash
# If you're sure you want to overwrite
git push -f origin main

# Better: pull and merge
git pull origin main --allow-unrelated-histories
```

### Change commit author
```bash
git config user.name "Your Name"
git config user.email "your.email@example.com"

# Amend last commit
git commit --amend --reset-author
```

---

## ✅ Repository Checklist

- [x] Git initialized
- [x] Initial commit created
- [x] .gitignore configured
- [x] README.md created
- [x] LICENSE added (MIT)
- [ ] GitHub repository created
- [ ] Remote origin added
- [ ] Pushed to GitHub
- [ ] Repository description added
- [ ] Topics/tags added on GitHub

---

## 📊 Repository Statistics

Current state:
- **24 files**
- **4,007 lines of code**
- **1 commit**
- **0 branches** (only main)
- **0 tags**

Ready to share with the world! 🎉

---

For more information, see:
- [GitHub Docs](https://docs.github.com)
- [Pro Git Book](https://git-scm.com/book)
- [Git Cheat Sheet](https://education.github.com/git-cheat-sheet-education.pdf)
