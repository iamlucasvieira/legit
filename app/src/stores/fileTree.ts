// src/stores/fileTree.ts
import { defineStore } from 'pinia'
import { invoke } from "@tauri-apps/api/core";

export interface FileNode {
  name: string
  path: string
  type: 'file' | 'directory'
  children?: FileNode[]
}

export const useFileTreeStore = defineStore('fileTree', {
  state: () => ({
    rootNodes: [] as FileNode[],
    expanded: new Set<string>(),
    selected: null as string | null,
  }),
  actions: {
    async loadDir(path: string) {
      // Only fetch once
      const node = this.findNode(path, this.rootNodes)
      if (node && node.children !== undefined) return

      const entries: Array<{ name: string; path: string; type: string }> =
        await invoke('list_files', { path })
      const children = entries.map(e => ({
        name: e.name,
        path: e.path,
        type: e.type === 'directory' ? 'directory' : 'file',
      })) as FileNode[]
      if (node) node.children = children
      else this.rootNodes = children
    },
    toggle(path: string) {
      if (this.expanded.has(path)) {
        this.expanded.delete(path)
      } else {
        this.expanded.add(path)
        this.loadDir(path)
      }
    },
    select(path: string) {
      this.selected = path
    },
    findNode(path: string, list: FileNode[]): FileNode | undefined {
      for (const n of list) {
        if (n.path === path) return n
        if (n.children) {
          const found = this.findNode(path, n.children)
          if (found) return found
        }
      }
    },
  },
})
