<template>
  <li>
    <div
      :class="['item', { selected: isSelected, directory: isDir }]"
      @click="onClick"
    >
      <span v-if="isDir" @click.stop="toggle">
        {{ isOpen ? "📂" : "📁" }}
      </span>
      <span>{{ node.name }}</span>
    </div>
    <ul v-if="isDir && isOpen" class="children">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
      />
    </ul>
  </li>
</template>

<script lang="ts">
import { defineComponent, computed } from "vue";
import { useFileTreeStore, FileNode } from "@/stores/fileTree";

export default defineComponent({
  name: "FileTreeNode",
  props: {
    node: { type: Object as () => FileNode, required: true },
  },
  setup(props) {
    const store = useFileTreeStore();
    const isDir = computed(() => props.node.type === "directory");
    const isOpen = computed(() => store.expanded.has(props.node.path));
    const isSelected = computed(() => store.selected === props.node.path);

    function toggle() {
      store.toggle(props.node.path);
    }
    function onClick() {
      if (isDir.value) toggle();
      else store.select(props.node.path);
    }

    return { isDir, isOpen, isSelected, toggle, onClick };
  },
});
</script>

<style scoped>
.file-tree,
.children {
  list-style: none;
  padding-left: 1rem;
}
.item {
  cursor: pointer;
  padding: 0.2rem 0.5rem;
}
.item.selected {
  background-color: #d0eaff;
}
.item.directory span:first-child {
  margin-right: 0.5rem;
}
</style>
