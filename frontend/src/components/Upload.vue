<script setup lang="ts">
import { ref } from 'vue';
import { useToast } from "primevue/usetoast";
import { Badge, Button, FileUpload, type FileUploadSelectEvent } from 'primevue';
import Divider from 'primevue/divider';
import ProgressSpinner from 'primevue/progressspinner';

const toast = useToast();

const uploading = ref(false);
const MAX_SIZE = 500 * 1024 * 1024; // 500MB

const uploadEvent = (callback: () => void) => {
  callback();
};

const onUploadHandler = () => {
  toast.add({ severity: "info", summary: "Success", detail: "File(s) Uploaded", life: 3000 });
  uploading.value = false
};

const onErrorHandler = () => {
  toast.add({ severity: "error", summary: "Failed", detail: "Something went wrong during file(s) upload", life: 3000 });
}

function onFileSelect(event: FileUploadSelectEvent) {
  event.files.forEach(file => {
    if (file.type.startsWith('video/')) {
      file.blobUrl = URL.createObjectURL(file);
    }
    else file.blobUrl = file.objectURL
  });
}

const maximize = (event) => {
  const video = event.target;
  if (video.requestFullscreen) {
    video.requestFullscreen();
  }
};

const onBeforeUploadHandler = () => {
  uploading.value = true
}
</script>


<template>
  <FileUpload url="/api/upload" @upload="onUploadHandler" @error="onErrorHandler" :maxFileSize=MAX_SIZE :multiple="true"
    accept="image/*,video/*" @select="onFileSelect" @before-upload="onBeforeUploadHandler">
    <template #header="{ chooseCallback, uploadCallback, clearCallback, files }">
      {{ uploader }}
      <div class="flex flex-wrap justify-content-between align-items-center flex-1 gap-0">
        <div class="flex gap-2">
          <Button @click="chooseCallback()" icon="pi pi-images" rounded outlined severity="secondary"
            :disabled="uploading"></Button>
          <Button @click="uploadEvent(uploadCallback)" icon="pi pi-cloud-upload" rounded outlined severity="success"
            :disabled="!files || files.length === 0 || uploading"></Button>
          <Button @click="clearCallback()" icon="pi pi-times" rounded outlined severity="danger"
            :disabled="!files || files.length === 0 || uploading"></Button>
        </div>
        <Divider />
      </div>
    </template>
    <template #content="{ files, uploadedFiles, removeFileCallback, removeUploadedFileCallback }">
      <div class="file-preview flex flex-wrap gap-2">
        <div v-for="(file, index) of uploadedFiles" :key="file.name + file.type + file.size"
          class="items-center flex flex-column border-1 p-2 border-round surface-border align-items-center max-w-9rem"
          width="100" height="50">
          <div class="flex-grow-1">
            <img v-if="file.type.startsWith('image/')" :src="file.blobUrl" alt="image preview" width="100%"
              @dblclick="maximize($event)" />
            <video v-else-if="file.type.startsWith('video/')" width="100%" :src="file.blobUrl" autoplay loop muted
              @dblclick="maximize($event)"></video>
          </div>
          <span class="font-semibold text-ellipsis max-w-60 whitespace-nowrap overflow-hidden">{{ file.name }}</span>
          <Badge value="Completed" severity="success" class="mb-1" />
          <Button icon="pi pi-times" @click="removeUploadedFileCallback(index)" variant="outlined" rounded
            severity="danger" />
        </div>

        <div v-for="(file, index) of files" :key="file.name + file.type + file.size"
          class="items-center flex flex-column border-1 p-2 border-round surface-border align-items-center max-w-9rem relative"
          width="100" height="50">
          <div v-if="uploading" class="spinner-overlay">
            <ProgressSpinner />
          </div>
          <div class="flex-grow-1">
            <img v-if="file.type.startsWith('image/')" :src="file.blobUrl" alt="image preview" width="100%"
              @dblclick="maximize($event)" />
            <video v-else-if="file.type.startsWith('video/')" width="100%" :src="file.blobUrl" autoplay loop muted
              @dblclick="maximize($event)"></video>
          </div>
          <span class="font-semibold text-ellipsis max-w-60 whitespace-nowrap overflow-hidden">{{ file.name }}</span>
          <Badge value="Pending" severity="warn" class="mb-1" />
          <Button icon="pi pi-times" @click="removeFileCallback(index)" variant="outlined" rounded severity="danger" />
        </div>
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

.spinner-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
</style>
