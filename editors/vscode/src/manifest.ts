import * as fs from 'fs';
import * as path from 'path';

export interface ManifestProfileEntry {
    label: string;
    description: string;
}

const DEFAULT_PROFILES: ManifestProfileEntry[] = [
    { label: 'debug', description: 'Development build profile with debugging symbols' },
    { label: 'release', description: 'Optimized build profile with bytecode inlining' },
    { label: 'test', description: 'Test build profile with unit testing harnesses enabled' },
];

const MANIFEST_FILENAMES = ['manifest.yaml', 'manifest.yml', 'manifest.toml'];

export function resolveManifestFile(directory: string): string | undefined {
    for (const filename of MANIFEST_FILENAMES) {
        const candidate = path.join(directory, filename);
        if (fs.existsSync(candidate)) {
            return candidate;
        }
    }
    return undefined;
}

export function findProjectRoot(initialFilePath: string): string | undefined {
    try {
        let currentDirectory = fs.statSync(initialFilePath).isDirectory()
            ? initialFilePath
            : path.dirname(initialFilePath);

        for (let depthIndex = 0; depthIndex < 10; depthIndex += 1) {
            if (resolveManifestFile(currentDirectory)) {
                return currentDirectory;
            }
            const parentDirectory = path.dirname(currentDirectory);
            if (parentDirectory === currentDirectory) {
                break;
            }
            currentDirectory = parentDirectory;
        }
    } catch {
        return undefined;
    }
    return undefined;
}

/**
 * Parses declared build profiles from manifest.yaml, manifest.yml, or manifest.toml.
 * Supports YAML `vars:` anchor definitions and `<<: *alias` merge keys.
 */
export function parseManifestProfiles(
    manifestContent: string,
    isYaml: boolean = true
): ManifestProfileEntry[] {
    const discovered: ManifestProfileEntry[] = [];
    const seen = new Set<string>();

    if (isYaml) {
        const lines = manifestContent.split(/\r?\n/);
        let profilesIndent = -1;
        let currentProfile: string | undefined;
        let currentDetails: string[] = [];

        const flushProfile = () => {
            if (currentProfile && !seen.has(currentProfile)) {
                seen.add(currentProfile);
                discovered.push({
                    label: currentProfile,
                    description:
                        currentDetails.length > 0
                            ? currentDetails.join(', ')
                            : defaultProfileDescription(currentProfile),
                });
            }
            currentProfile = undefined;
            currentDetails = [];
        };

        for (const rawLine of lines) {
            const trimmed = rawLine.trim();
            if (!trimmed || trimmed.startsWith('#')) {
                continue;
            }
            const indent = rawLine.length - rawLine.trimStart().length;

            if (profilesIndent >= 0 && indent <= profilesIndent) {
                flushProfile();
                profilesIndent = -1;
            }

            if (trimmed === 'profiles:' || trimmed.startsWith('profiles:')) {
                profilesIndent = indent;
                continue;
            }

            if (profilesIndent >= 0) {
                if (indent === profilesIndent + 2 && trimmed.endsWith(':')) {
                    const keyRaw = trimmed.slice(0, -1).trim();
                    const keyName = keyRaw.split(/\s+/)[0];
                    if (keyName && !keyName.startsWith('&') && keyName !== '<<') {
                        flushProfile();
                        currentProfile = keyName;
                    }
                } else if (currentProfile && indent >= profilesIndent + 4) {
                    const kvMatch = trimmed.match(/^([a-zA-Z0-9_]+)\s*:\s*(.+)$/);
                    if (kvMatch && kvMatch[1] && kvMatch[2]) {
                        const fieldKey = kvMatch[1];
                        const fieldVal = kvMatch[2].replace(/#.*$/, '').trim();
                        if (fieldKey === 'description' || fieldKey === 'build_type' || fieldKey === 'optimization') {
                            currentDetails.push(`${fieldKey}: ${fieldVal.replace(/^["']|["']$/g, '')}`);
                        }
                    }
                }
            }
        }
        flushProfile();
    } else {
        const profileHeaderRegex = /^\[profiles\.([a-zA-Z0-9_\-]+)\]/gm;
        let match: RegExpExecArray | null;
        while ((match = profileHeaderRegex.exec(manifestContent)) !== null) {
            const profileName = match[1];
            if (profileName && !seen.has(profileName)) {
                seen.add(profileName);
                discovered.push({
                    label: profileName,
                    description: defaultProfileDescription(profileName),
                });
            }
        }
    }

    if (discovered.length === 0) {
        return [...DEFAULT_PROFILES];
    }
    return discovered;
}

function defaultProfileDescription(profileName: string): string {
    const preset = DEFAULT_PROFILES.find((item) => item.label === profileName);
    if (preset) {
        return preset.description;
    }
    return `Custom manifest build profile '${profileName}'`;
}

/**
 * Resolves the compiled package artifact path under `build/<profile>/`,
 * preferring `.slxp` with `.slxarc` compatibility fallback.
 */
export function resolveArtifactPath(
    projectRoot: string,
    profileName: string,
    packageName: string
): string {
    const slxpPath = path.join(projectRoot, 'build', profileName, `${packageName}.slxp`);
    if (fs.existsSync(slxpPath)) {
        return slxpPath;
    }
    const slxarcPath = path.join(projectRoot, 'build', profileName, `${packageName}.slxarc`);
    if (fs.existsSync(slxarcPath)) {
        return slxarcPath;
    }
    return slxpPath;
}

/**
 * Recursively collects all `.slx` source files inside a directory.
 */
export function collectSolixSourcesRecursive(directory: string): string[] {
    if (!fs.existsSync(directory)) {
        return [];
    }
    const collected: string[] = [];
    const entries = fs.readdirSync(directory, { withFileTypes: true });
    for (const entry of entries) {
        const fullPath = path.join(directory, entry.name);
        if (entry.isDirectory()) {
            collected.push(...collectSolixSourcesRecursive(fullPath));
        } else if (entry.isFile() && entry.name.endsWith('.slx')) {
            collected.push(fullPath);
        }
    }
    collected.sort();
    return collected;
}

/**
 * Builds profile-routed CLI argument arrays for `build`, `run`, and `test`.
 */
export function buildProfileCommandArgs(
    action: 'build' | 'run' | 'test',
    profileName: string,
    artifactOrTarget?: string
): string[] {
    if (action === 'build') {
        return ['build', '--profile', profileName];
    }
    if (action === 'test') {
        return artifactOrTarget
            ? ['run', '--profile', profileName, '--test', artifactOrTarget]
            : ['run', '--profile', profileName, '--test'];
    }
    return artifactOrTarget
        ? ['run', '--profile', profileName, artifactOrTarget]
        : ['run', '--profile', profileName];
}
