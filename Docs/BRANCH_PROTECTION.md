# Branch Protection Rules

This document describes the branch protection rules that should be configured manually for the `main` branch in the MyAI repository.

## Required Branch Protection Settings

### Protect matching branches

- **Branch name pattern:** `main`

### Pull Request Settings

- **Require a pull request before merging:** ✅ Enabled
  - **Require approvals:** 1 minimum
  - **Dismiss stale pull request approvals when new commits are pushed:** ✅ Enabled
  - **Require review from Code Owners:** ✅ Enabled (if CODEOWNERS file exists)

- **Require status checks to pass before merging:** ✅ Enabled
  - **Require branches to be up to date before merging:** ✅ Enabled
  - **Status checks that are required:**
    - `Backend Tests` (from CI workflow)
    - `Frontend Tests` (from CI workflow)

- **Require conversation resolution before merging:** ✅ Enabled

- **Require signed commits:** ✅ Recommended

- **Require linear history:** ✅ Recommended (optional)

- **Require merge queue:** ❌ Disabled (optional, can be enabled for high-traffic repos)

### Push Restrictions

- **Restrict who can push to matching branches:** ✅ Enabled
  - Only repository administrators and the CI service account should have direct push access.

- **Allow force pushes:** ❌ Disabled

- **Allow deletions:** ❌ Disabled

### Rules applied to everyone including administrators

- **Do not bypass the above rules:** ✅ Enabled (recommended)

## GitHub Actions Secrets Required

The following secrets must be configured in the repository settings for the CI/CD pipelines to function:

| Secret Name | Description | Example |
|-------------|-------------|---------|
| `HARBOR_HOST` | Harbor registry hostname | `harbor.example.com:8003` |
| `HARBOR_USER` | Harbor registry username | `admin` |
| `HARBOR_PASS` | Harbor registry password | `********` |
| `KUBECONFIG` | Base64-encoded kubeconfig for cluster access | `base64_encoded_string` |
| `K8S_NAMESPACE` | Kubernetes namespace for deployment | `myai-production` |

## How to Configure

1. Go to **Settings** → **Branches** in the GitHub repository.
2. Click **Add rule** next to "Branch protection rules".
3. Enter `main` as the branch name pattern.
4. Configure the settings as described above.
5. Click **Create** or **Save changes**.

## Verification

After configuration:
- Direct pushes to `main` should be blocked.
- PRs require at least 1 approval.
- PRs require all status checks to pass.
- The CI workflow runs on every PR to `main`.
