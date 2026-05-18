import { randomBytes } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { cancel, intro, isCancel, outro, spinner, text } from '@clack/prompts';
import { Command } from 'commander';
import fs from 'fs-extra';

type Project = {
    kebab: string;
    snake: string;
    title: string;
    authSecret: string;
};

const packageRoot = path.resolve(fileURLToPath(import.meta.url), '..', '..');
const repoRoot = path.resolve(packageRoot, '..', '..');
const bundledTemplateDir = path.join(packageRoot, 'templates', 'default');
const monorepoTemplateDir = path.join(repoRoot, 'templates', 'default');
const textExtensions = new Set([
    '',
    '.dockerignore',
    '.gitignore',
    '.lock',
    '.md',
    '.rs',
    '.sql',
    '.toml',
    '.txt',
    '.yml',
    '.yaml',
]);

export async function run(argv: string[]) {
    const program = new Command()
        .name('create-rust-starter')
        .description('Create a production-ready Axum backend from the Rust Starter template.')
        .argument('[project-name]', 'directory name for the new project')
        .option('--no-install', 'reserved for future dependency install support')
        .action(async (projectName: string | undefined) => {
            await createProject(projectName);
        });

    await program.parseAsync(argv);
}

async function createProject(projectName: string | undefined) {
    intro('create-rust-starter');

    const chosenName = projectName ?? (await promptProjectName());
    const project = normalizeProject(chosenName);
    const targetDir = path.resolve(process.cwd(), chosenName);
    const templateDir = resolveTemplateDir();

    if (!templateDir) {
        cancel('Template directory not found.');
        throw new Error('template directory not found');
    }

    if (await fs.pathExists(targetDir)) {
        cancel(`Target directory already exists: ${targetDir}`);
        throw new Error(`target directory already exists: ${targetDir}`);
    }

    const s = spinner();
    s.start(`Creating ${project.kebab}`);
    await copyTemplate(templateDir, targetDir, project);
    s.stop(`Created ${project.kebab}`);

    outro(`Next steps:
  cd ${chosenName}
  cp config.example.yml config.yml
  docker compose up -d
  cargo xtask migrate up
  cargo run`);
}

async function promptProjectName() {
    const value = await text({
        message: 'Project name',
        placeholder: 'my-api',
        validate(input) {
            return normalizeName(input) ? undefined : 'Use letters, numbers, dashes, or underscores.';
        },
    });

    if (isCancel(value)) {
        cancel('Operation cancelled.');
        process.exit(0);
    }

    return value;
}

function normalizeProject(input: string): Project {
    const kebab = normalizeName(path.basename(input));

    if (!kebab) {
        throw new Error(`invalid project name: ${input}`);
    }

    return {
        kebab,
        snake: kebab.replace(/-/g, '_'),
        title: kebab
            .split('-')
            .filter(Boolean)
            .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
            .join(' '),
        authSecret: randomBytes(32).toString('hex'),
    };
}

function normalizeName(input: string) {
    return input
        .trim()
        .replace(/[^a-zA-Z0-9_-]+/g, '-')
        .replace(/_+/g, '-')
        .replace(/-+/g, '-')
        .replace(/^-|-$/g, '')
        .toLowerCase();
}

function resolveTemplateDir() {
    if (fs.existsSync(bundledTemplateDir)) {
        return bundledTemplateDir;
    }

    if (fs.existsSync(monorepoTemplateDir)) {
        return monorepoTemplateDir;
    }

    return null;
}

async function copyTemplate(sourceDir: string, targetDir: string, project: Project) {
    await fs.ensureDir(targetDir);

    const entries = await fs.readdir(sourceDir, { withFileTypes: true });

    for (const entry of entries) {
        const sourcePath = path.join(sourceDir, entry.name);
        const targetPath = path.join(targetDir, entry.name);

        if (entry.isDirectory()) {
            await copyTemplate(sourcePath, targetPath, project);
            continue;
        }

        if (!entry.isFile()) {
            continue;
        }

        await copyFileWithReplacements(sourcePath, targetPath, project);
    }
}

async function copyFileWithReplacements(sourcePath: string, targetPath: string, project: Project) {
    const extension = path.extname(sourcePath);

    if (!textExtensions.has(extension)) {
        await fs.copy(sourcePath, targetPath);
        return;
    }

    const original = await fs.readFile(sourcePath, 'utf8');
    const replaced = original
        .replaceAll('development-auth-secret-change-me-please', project.authSecret)
        .replaceAll('rust_starter', project.snake)
        .replaceAll('rust-starter', project.kebab)
        .replaceAll('Rust Starter', project.title);

    await fs.outputFile(targetPath, replaced);
}
