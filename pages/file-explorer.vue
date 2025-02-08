<template>
  <div class="flex justify-center items-center min-h-[calc(100vh-45px)] bg-gray-100 w-full">
    <div class="flex flex-col w-full h-full px-6 pt-6">
      <h1 class="text-2xl font-bold text-purple-700 mb-6 text-center">DragonNest Converter</h1>
      <div class="bg-white shadow-lg rounded-lg p-8">
        <div class="mb-4">
          <label class="block text-gray-700">Open Mode</label>
          <select
            class="mt-1 block w-full bg-gray-50 border text-black border-gray-300 rounded-md shadow-sm focus:ring-purple-500 focus:border-purple-500"
            v-model="openMode">
            <option>Single File</option>
            <option>Folder</option>
          </select>
        </div>

        <div class="mb-4">
          <label class="block text-gray-700">Convert Mode</label>
          <select
            class="mt-1 block w-full bg-gray-50 border text-black border-gray-300 rounded-md shadow-sm focus:ring-purple-500 focus:border-purple-500"
            v-model="convertMode">
            <option>Convert to .tsv</option>
            <option>Convert to .dnt</option>
            <option>Convert act v6 to v5</option>
            <option>Extract Pak</option>
          </select>
        </div>

        <div class="mb-4">
          <label class="block text-gray-700">Input</label>
          <div class="flex">
            <input type="text"
              class="flex-grow mt-[1.25rem] block w-full bg-gray-50 border border-gray-300 rounded-l-md shadow-sm focus:ring-purple-500 focus:border-purple-500 text-black"
              v-model="inputpath" readonly>
            <button class="bg-purple-600 text-white px-4 py-2 rounded-r-md" @click="openFileDialog">Browse</button>
          </div>
        </div>

        <div class="mb-4">
          <label class="block text-gray-700">Output</label>
          <div class="flex">
            <input type="text"
              class="flex-grow mt-[1.25rem]  block w-full bg-gray-50 border border-gray-300 rounded-l-md shadow-sm focus:ring-purple-500 focus:border-purple-500 text-black"
              v-model="outputpath" readonly>
            <button class="bg-purple-600 text-white px-4 py-2 rounded-r-md" @click="outputFileDialog">Browse</button>
          </div>
        </div>

        <div class="mb-4" v-if="convertMode == 'Extract Pak'">
          <label class="block text-gray-700">
            <input type="checkbox" v-model="usingEncryption" class="mr-2">
            Using Encryption?
          </label>
        </div>
        <button class="w-full bg-purple-700 text-white py-2 rounded-md hover:bg-purple-800 transition"
          @click="convert">Convert</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

const inputpath = ref('');
const outputpath = ref('');
const openMode = ref('Single File');
const convertMode = ref('Convert to .tsv');
const usingEncryption = ref(false);

const openFileDialog = async () => {
  const file = await open({
    multiple: false,
    directory: openMode.value == "Folder" ? true : false,
    filters: convertMode.value === "Convert to .tsv"
      ? [{ name: 'DNT Files', extensions: ['dnt'] }]
      : convertMode.value === "Convert to .dnt"
      ? [{ name: 'TSV Files', extensions: ['tsv'] }]
      : convertMode.value === "Extract Pak"
      ? [{ name: 'PAK Files', extensions: ['pak'] }]
      : [{ name: 'ACT Files', extensions: ['act'] }]
});

const extension =
    convertMode.value === "Convert to .tsv"
      ? "\\*.dnt"
      : convertMode.value === "Convert to .dnt"
      ? "\\*.tsv"
      : convertMode.value === "Extract Pak"
      ? "\\*.pak"
      : "";

  inputpath.value = openMode.value === "Folder" ? file + extension : file;
};

const outputFileDialog = async () => {
  if (openMode.value == "Folder") {
    const file = await open({
      multiple: false,
      directory: true,
    });
    outputpath.value = file;
  } else {
    if (convertMode.value === "Extract Pak") {
      const file = await open({
        multiple: false,
        directory: true,
      });
      outputpath.value = file;
    } else {
      const file = await save({
        multiple: false,
        directory: false,
        defaultPath: convertMode.value === "Convert to .tsv"
          ? inputpath.value.replace(/\.(dnt|act)$/, ".tsv")
          : convertMode.value === "Convert to .dnt"
          ? inputpath.value.replace(/\.(tsv|act)$/, ".dnt")
          : inputpath.value.replace(/\.(tsv|dnt)$/, ".act"),
        filters: convertMode.value === "Convert to .tsv"
          ? [{ name: 'TSV Files', extensions: ['tsv'] }]
          : convertMode.value === "Convert to .dnt"
          ? [{ name: 'DNT Files', extensions: ['dnt'] }]
          : [{ name: 'ACT Files', extensions: ['act'] }]
      });
      outputpath.value = file;
    }
  }
};

const convert = async () => {
  invoke('convert', { input_file: inputpath.value, output_file: outputpath.value, open_mode: openMode.value, convert_mode: convertMode.value, encryption: usingEncryption.value });
};

watch([openMode, convertMode], () => {
  inputpath.value = '';
  outputpath.value = '';
});

</script>

<style scoped>
h1 {
  font-size: 24px;
  margin-bottom: 20px;
}

button {
  margin-top: 20px;
  padding: 10px 20px;
  font-size: 16px;
}
</style>