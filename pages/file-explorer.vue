<style>
  body {
      font-family: Arial, sans-serif;
      background-color: #f0f0f0;
      margin: 0;
      padding: 0;
      display: flex;
      justify-content: center;
      align-items: center;
      height: 100vh;
  }
  .container {
      background-color: #e0e0e0;
      border: 1px solid #ccc;
      padding: 20px;
      width: 600px; /* Increased width */
      box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
  }
  .header {
      background-color: #800080;
      color: white;
      padding: 10px;
      text-align: left;
      font-size: 16px;
  }
  .form-group {
      margin-bottom: 15px;
  }
  .form-group label {
      display: block;
      margin-bottom: 5px;
  }
  .form-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 15px;
  }
  .form-row label {
      margin-right: 10px;
  }
  .form-row select {
      width: calc(50% - 40px);
      padding: 5px;
  }
  .input-group, .output-group {
      display: flex;
      align-items: center;
      justify-content: space-between;
  }
  .input-group input[type="text"], .output-group input[type="text"] {
      width: 400px;
      padding: 5px;
      margin-right: 10px;
  }
  .input-group button, .output-group button {
      padding: 5px 10px;
  }
  .input-group .browse-button, .output-group .browse-button {
      width: 70px;
  }
  .convert-button {
      text-align: center;
  }
  .convert-button button {
      padding: 10px 20px;
  }
</style>
<template>
  <div class="container">
      <div class="header">DNTableConverter</div>
      <div class="form-group">
          <label>Option</label>
          <div class="form-row">
              <label>Open Mode</label>
              <select v-model="openMode">
                  <option selected>Single File</option>
                  <option>Folder</option>
              </select>
              <label>Convert Mode</label>
              <select v-model="convertMode" >
                  <option>Convert to .tsv</option>
                  <option>Convert to .dnt</option>
              </select>
          </div>
      </div>
      <div class="form-group">
          <div class="input-group">
              <label>Input</label>
              <input type="text" v-model="a" readonly>
              <button @click="openFileDialog" class="browse-button">Browse</button>
          </div>
      </div>
      <div class="form-group">
          <div class="output-group">
              <label>Output</label>
              <input type="text" v-model="b" readonly>
              <button @click="outputFileDialog" class="browse-button">Browse</button>
          </div>
      </div>
      <div class="convert-button">
          <button @click="convert">Convert</button>
      </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

const a = ref('');
const b = ref('');
const openMode = ref('Single File');
const convertMode = ref('Convert to .tsv');

const openFileDialog = async () => {
  const file = await open({
    multiple: false,
    directory: openMode.value == "Folder" ? true : false,
    filters: convertMode.value == "Convert to .tsv" ? [{ name:'.dnt' , extensions: ['dnt'] }] : [{ name:'.tsv' , extensions: ['tsv'] }]
  });
  a.value = openMode.value == "Folder" ? file+ "\\*.dnt" : file;
};

const outputFileDialog = async () => {
  if (openMode.value == "Folder") {
    const file = await open({
      multiple: false,
      directory: true,
    });
    b.value = file;
  } else {
    const file = await save({
      multiple: false,
      directory: false,
      defaultPath: convertMode.value == "Convert to .tsv" ? a.value.replace(".dnt",".tsv") : a.value.replace(".tsv",".dnt"),
      filters: convertMode.value == "Convert to .tsv" ? [{ name:'.tsv' , extensions: ['tsv'] }] : [{ name:'.dnt' , extensions: ['dnt'] }]
    });
    b.value = file;
  }
};

const convert = async () => {
  invoke('convert', { input_file: a.value, output_file: b.value, open_mode: openMode.value, convert_mode: convertMode.value });
};

watch([openMode, convertMode], () => {
  a.value = '';
  b.value = '';
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