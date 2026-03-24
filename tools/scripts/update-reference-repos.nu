#!/usr/bin/env nu

# Update repository references from project manifest
#
# This script reads the project-manifest.yaml file and updates
# all repositories listed in the references->repos section
# in the .tmp/code directory. It clones missing repos and pulls
# latest changes for existing ones.
#
# Examples:
#   update-reference-repos.nu              # Clone missing repos and update existing ones
#   update-reference-repos.nu --clean      # Remove existing repos before cloning
#   update-reference-repos.nu --dry-run    # Show what would be done without executing

use lib/ui.nu *

def main [
    --clean (-c)     # Remove existing repos before cloning
    --dry-run (-d)   # Show what would be done without executing
]: nothing -> nothing {
    let manifest_path = "project-manifest.yaml"

    # Check if manifest file exists
    if not ($manifest_path | path exists) {
        error $"Manifest file not found: ($manifest_path)"
        exit 1
    }

    header "📦 Reference Repository Update"

    # Read and parse the manifest file
    let manifest = open $manifest_path
    let clone_dir = $manifest.references.clone_dir? | default ".tmp/code"
    let repos = $manifest.references.repos?

    if $repos == null or ($repos | length) == 0 {
        warning "No repositories found in manifest"
        exit 0
    }

    info $"Found ($repos | length) repositories to process"
    divider

    # Create clone directory if it doesn't exist
    if not ($clone_dir | path exists) {
        if $dry_run {
            info $"[DRY RUN] Would create directory: ($clone_dir)"
        } else {
            mkdir $clone_dir
            success $"Created directory: ($clone_dir)"
        }
    }

    # Process each repository
    for repo in $repos {
        process_repo $repo $clone_dir $clean $dry_run
    }

    divider "done"
}

# Process a single repository
def process_repo [
    repo: record,
    clone_dir: string,
    clean: bool,
    dry_run: bool
]: nothing -> nothing {
    let repo_path = $"($clone_dir)/($repo.name)"

    section $"󰊢 ($repo.name)"

    # Handle clean flag
    if $clean and ($repo_path | path exists) {
        if $dry_run {
            info $"[DRY RUN] Would remove existing: ($repo_path)"
        } else {
            rm -rf $repo_path
            success "Removed existing repository"
        }
    }

    # Clone or update repository
    if ($repo_path | path exists) {
        # Repository exists, fetch updates
        if $dry_run {
            info $"[DRY RUN] Would fetch updates and checkout branch: ($repo.branch)"
        } else {
            # Fetch and update
            do -i { gum spin --spinner.foreground="212" --title=$"Updating from origin/($repo.branch)" -- sh -c $"cd ($repo_path) && git fetch origin && git checkout ($repo.branch) && git pull origin ($repo.branch) 2>&1" }

            if $env.LAST_EXIT_CODE == 0 {
                success $"Updated to latest ($repo.branch)"
            } else {
                error "Failed to update"
            }
        }
    } else {
        # Repository doesn't exist, clone it
        if $dry_run {
            info $"[DRY RUN] Would clone from: ($repo.url) on branch ($repo.branch)"
        } else {
            do -i { gum spin --spinner.foreground="212" --title=$"Cloning branch ($repo.branch)" -- sh -c $"git clone -b ($repo.branch) ($repo.url) ($repo_path) 2>&1" }

            if $env.LAST_EXIT_CODE == 0 {
                success "Cloned successfully"
            } else {
                error "Failed to clone"
            }
        }
    }
}

# Display status of all repositories
def "main status" []: nothing -> table {
    let manifest_path = "project-manifest.yaml"

    # Check if manifest file exists
    if not ($manifest_path | path exists) {
        error $"Manifest file not found: ($manifest_path)"
        exit 1
    }

    header "📊 Repository Status"

    # Read manifest
    let manifest = open $manifest_path
    let clone_dir = $manifest.references.clone_dir? | default ".tmp/code"
    let repos = $manifest.references.repos?

    if $repos == null or ($repos | length) == 0 {
        warning "No repositories found in manifest"
        exit 0
    }

    # Check status of each repo
    let status = $repos | each { |repo|
        let repo_path = $"($clone_dir)/($repo.name)"
        let exists = ($repo_path | path exists)

        let current_branch = if $exists {
            (do -i {
                cd $repo_path
                ^git branch --show-current
            } | complete | get stdout | str trim)
        } else {
            ""
        }

        let status = if $exists {
            if $current_branch == $repo.branch { "✓ Synced" } else { "⚠ Different branch" }
        } else {
            "✗ Not cloned"
        }

        {
            name: $repo.name
            branch: $repo.branch
            current: $current_branch
            status: $status
        }
    }

    $status | table --index false
}
