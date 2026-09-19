import { get } from 'svelte/store';
import { 
  videoFilePath, 
  trimStart, 
  trimEnd, 
  exportFormat, 
  exportQuality, 
  autoSnap,
  videoMetadata,
  previewQuality
} from './store';
import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';

export interface ProjectData {
  videoFilePath: string;
  trimStart: number;
  trimEnd: number;
  exportFormat: string;
  exportQuality: number;
  autoSnap: boolean;
  previewQuality: number;
  metadata?: any;
}

export async function saveProject() {
  const currentVideoPath = get(videoFilePath);
  if (!currentVideoPath) {
    throw new Error("No video loaded to save.");
  }

  const projectData: ProjectData = {
    videoFilePath: currentVideoPath,
    trimStart: get(trimStart),
    trimEnd: get(trimEnd),
    exportFormat: get(exportFormat),
    exportQuality: get(exportQuality),
    autoSnap: get(autoSnap),
    previewQuality: get(previewQuality),
    metadata: get(videoMetadata),
  };

  const savePath = await save({
    filters: [
      { name: 'Cut Project', extensions: ['cutterproj', 'json'] },
      { name: 'All Files', extensions: ['*'] }
    ],
    defaultPath: 'untitled_project.cutterproj'
  });

  if (savePath) {
    await writeTextFile(savePath, JSON.stringify(projectData, null, 2));
    return savePath;
  }
  return null;
}

export async function loadProjectFromFile(filePath: string): Promise<ProjectData | null> {
  const content = await readTextFile(filePath);
  try {
    const projectData: ProjectData = JSON.parse(content);
    if (!projectData.videoFilePath) {
      throw new Error("Invalid project file: No video file path found.");
    }
    return projectData;
  } catch (e) {
    throw new Error(`Failed to parse project file: ${e}`);
  }
}

export async function loadProjectData(): Promise<ProjectData | null> {
  const openPath = await open({
    multiple: false,
    filters: [
      { name: 'Cut Project', extensions: ['cutterproj', 'json'] },
      { name: 'All Files', extensions: ['*'] }
    ]
  });

  if (openPath && typeof openPath === 'string') {
    return loadProjectFromFile(openPath);
  }
  return null;
}

