<template>
  <ul class="file-tree">
    <FileTreeNode
      v-for="node in store.rootNodes"
      :key="node.path"
      :node="node"
    />
  </ul>
</template>

<script lang="ts">
import { defineComponent, onMounted } from "vue";
import { useFileTreeStore } from "../stores/fileTree";
import FileTreeNode from "./FileTreeNode.vue";
import * as path from "@tauri-apps/api/path";

export default defineComponent({
  components: { FileTreeNode },
  setup() {
    const store = useFileTreeStore();

    // load the project root on mount
    onMounted(async () => {
      const home = await path.homeDir();
      store.loadDir(home);
    });

    return { store };
  },
});
</script>
