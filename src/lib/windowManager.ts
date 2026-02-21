import {
  getCurrentWindow,
  PhysicalSize,
  PhysicalPosition,
} from "@tauri-apps/api/window";
import { currentMonitor } from "@tauri-apps/api/window";

const DEFAULT_WIDTH = 1400;
const DEFAULT_HEIGHT = 900;
const TASKBAR_WIDTH = 420;
const TASKBAR_HEIGHT = 52;
const EXPANDED_TASKBAR_HEIGHT = 380;

/**
 * Shrinks the window to the compact taskbar and centers it above the system taskbar.
 */
export async function shrinkToTaskbar() {
  const win = getCurrentWindow();
  const monitor = await currentMonitor();
  const screenW = monitor?.size.width ?? 1920;
  const screenH = monitor?.size.height ?? 1080;
  const scaleFactor = monitor?.scaleFactor ?? 1;

  const physW = Math.round(TASKBAR_WIDTH * scaleFactor);
  const physH = Math.round(TASKBAR_HEIGHT * scaleFactor);

  const marginY = 60; // System taskbar margin

  // Centered horizontally, bottom
  const x = Math.round((screenW - physW) / 2);
  const y = screenH - physH - marginY;

  await win.setSize(new PhysicalSize(physW, physH));
  await win.setPosition(new PhysicalPosition(x, y));
  await win.setAlwaysOnTop(true);
  await win.setDecorations(false);
  await win.setResizable(false);
}

/**
 * Expands the taskbar upward without moving its bottom edge.
 */
export async function expandTaskbar() {
  const win = getCurrentWindow();
  const monitor = await currentMonitor();
  const screenW = monitor?.size.width ?? 1920;
  const screenH = monitor?.size.height ?? 1080;
  const scaleFactor = monitor?.scaleFactor ?? 1;

  const physW = Math.round(TASKBAR_WIDTH * scaleFactor);
  const physH = Math.round(EXPANDED_TASKBAR_HEIGHT * scaleFactor);

  const marginY = 60;
  
  // Centered horizontally, bottom
  const x = Math.round((screenW - physW) / 2);
  const y = screenH - physH - marginY;

  await win.setSize(new PhysicalSize(physW, physH));
  await win.setPosition(new PhysicalPosition(x, y));
  await win.setAlwaysOnTop(true);
  await win.setDecorations(false);
  await win.setResizable(false);
}

/**
 * Collapses the expanded taskbar back to its compact size.
 */
export async function collapseTaskbar() {
  await shrinkToTaskbar();
}

/**
 * Restores the window to its default (start-screen) size and centers it.
 */
export async function restoreDefaultWindow() {
  const win = getCurrentWindow();
  const monitor = await currentMonitor();
  const screenW = monitor?.size.width ?? 1920;
  const screenH = monitor?.size.height ?? 1080;
  const scaleFactor = monitor?.scaleFactor ?? 1;

  const physW = Math.round(DEFAULT_WIDTH * scaleFactor);
  const physH = Math.round(DEFAULT_HEIGHT * scaleFactor);

  const x = Math.round((screenW - physW) / 2);
  const y = Math.round((screenH - physH) / 2);

  await win.setAlwaysOnTop(false);
  await win.setDecorations(true);
  await win.setResizable(true);
  await win.setSize(new PhysicalSize(physW, physH));
  await win.setPosition(new PhysicalPosition(x, y));
}
