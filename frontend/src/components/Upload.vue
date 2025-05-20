<script setup lang="ts">
import { ref } from 'vue';
import Dialog from 'primevue/dialog';
import Button from "primevue/button"
import FileUpload, { type FileUploadErrorEvent } from 'primevue/fileupload';
import { useToast } from "primevue/usetoast";

const toast = useToast();
const visible = ref(false);

const onAdvancedUpload = () => {
  toast.add({ severity: 'info', summary: 'Success', detail: 'File Uploaded', life: 3000 });
};

const onError = (event: FileUploadErrorEvent) => {
  toast.add({ severity: 'error', summary: 'Error', detail: `Something went wrong: ${event.xhr.statusText}`, life: 3000 });

}

</script>

<template>
  <div class="floating-btn">
    <Button icon="pi pi-plus" severity="success" variant="text" raised rounded v-tooltip="'Add a new image'"
      size="large" @click="visible = true" />
  </div>

  <Dialog header="Add a new image" modal v-model:visible="visible">
    <FileUpload name="demo[]" url="/api/upload" @error="onError" @upload="onAdvancedUpload" :multiple="true"
      accept="image/*">
      <template #empty>
        <span>Drag and drop files to here to upload.</span>
      </template>
    </FileUpload>



  </Dialog>
</template>

<style>
.floating-btn {
  position: fixed;
  bottom: 24px;
  right: 24px;
}
</style>
