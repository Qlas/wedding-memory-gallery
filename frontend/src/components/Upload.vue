<script setup lang="ts">
import { ref } from 'vue';
import { usePrimeVue } from 'primevue/config';
import { useToast } from "primevue/usetoast";
import { Badge, Button, FileUpload, ProgressBar } from 'primevue';

const $primevue = usePrimeVue();
const toast = useToast();

const totalSize = ref(0);
const totalSizePercent = ref(0);
const files = ref([]);
const uploader = ref();
const MAX_SIZE = 100 * 1024 * 1024; // 100MB

const onRemoveTemplatingFile = (file, removeFileCallback, index) => {
  removeFileCallback(index);
  totalSize.value -= parseInt(formatSize(file.size));
  totalSizePercent.value = totalSize.value / 10;
};

const onSelectedFiles = (event) => {
  event.files.forEach(file => {
    if (file.size > MAX_SIZE) {
      toast.add({
        severity: 'error',
        summary: 'File too large',
        detail: `The file "${file.name}" exceeds the ${formatSize(MAX_SIZE)} size limit.`,
        life: 4000
      });
      uploader.value.remove(event);
      return
    }
  });

  files.value = event.files;
  files.value.forEach((file) => {
    totalSize.value += parseInt(formatSize(file.size));
  });
};

const uploadEvent = (callback) => {
  totalSizePercent.value = totalSize.value / 10;
  callback();
};

const onTemplatedUpload = () => {
  toast.add({ severity: "info", summary: "Success", detail: "File Uploaded", life: 3000 });
};

const formatSize = (bytes) => {
  const k = 1024;
  const dm = 3;
  const sizes = $primevue.config.locale.fileSizeTypes;

  if (bytes === 0) {
    return `0 ${sizes[0]}`;
  }

  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const formattedSize = parseFloat((bytes / Math.pow(k, i)).toFixed(dm));

  return `${formattedSize} ${sizes[i]}`;
};
</script>


<template>
  <FileUpload ref="uploader" name="abc" url="/api/upload" @upload="onTemplatedUpload($event)" :multiple="true"
    :pt="{ ProgressBar: { root: '!hidden' } }" accept="image/*" @select="onSelectedFiles">
    <template #header="{ chooseCallback, uploadCallback, clearCallback, files }">
      <div class="flex flex-wrap justify-content-between align-items-center flex-1 gap-3">
        <div class="flex gap-2">
          <Button @click="chooseCallback()" icon="pi pi-images" rounded outlined severity="secondary"></Button>
          <Button @click="uploadEvent(uploadCallback)" icon="pi pi-cloud-upload" rounded outlined severity="success"
            :disabled="!files || files.length === 0"></Button>
          <Button @click="clearCallback()" icon="pi pi-times" rounded outlined severity="danger"
            :disabled="!files || files.length === 0"></Button>
        </div>
        <ProgressBar :value="totalSizePercent" :showValue="false" class="h-1 w-full">
          <span class="whitespace-nowrap">{{ totalSize }}B / 1Mb</span>
        </ProgressBar>
      </div>
    </template>
    <template #empty>
      <div class="flex align-items-center justify-content-center flex-column">
        <i class="pi pi-cloud-upload border-2 border-circle p-5 text-5xl text-400" />
        <p class="mt-4 mb-0">Drag and drop files to here to upload.</p>
      </div>
    </template>
  </FileUpload>
</template>


<style>
.card {
  background: var(--card-bg);
  border: var(--card-border);
  padding: 2rem;
  border-radius: 10px;
  margin-bottom: 1rem;
}

:root {
  --body-bg: var(--p-surface-50);
  --body-text-color: var(--p-surface-900);
  --card-border: 1px solid var(--border-color);
  --card-bg: var(--p-surface-0);
  --border-color: var(--p-surface-200);
  --text-color: var(--p-surface-700);
  --overlay-background: #ffffff;
}

:root[class='p-dark'] {
  --body-bg: var(--p-surface-950);
  --body-text-color: var(--p-surface-50);
  --card-border: 1px solid transparent;
  --card-bg: var(--p-surface-900);
  --border-color: rgba(255, 255, 255, 0.1);
  --text-color: var(--p-surface-0);
  --overlay-background: var(--p-surface-900);
}

html {
  font-size: 14px;
  line-height: normal;
}

body {
  margin: 0px;
  min-height: 100%;
  overflow-x: hidden;
  overflow-y: auto;
  background-color: var(--body-bg);
  font-weight: normal;
  color: var(--body-text-color);
  padding: 1rem;
}
</style>
