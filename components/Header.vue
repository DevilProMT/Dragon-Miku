<template>
        <header data-tauri-drag-region
        class="w-auto h-[45px] shrink-0 [background:linear-gradient(270deg,rgba(0,0,0,0.60)_0%,rgba(102,102,102,0.60)_48%)] border-b-white border-b border-solid flex items-center z-30">
        <div data-tauri-drag-region class="inline-flex items-center gap-2 mt-[5px] ml-[10px]">
            <img class="w-[37.549px] h-[35px]" alt="Logo"
                src="https://upload.wikimedia.org/wikipedia/en/a/a5/DragonNestLogo.png?20110427204833" />
            <p class="text-white [font-family:Almarai] text-sm font-bold leading-[normal]">
                Dragon Nest
            </p>
        </div>

        <!-- Navigation -->
        <nav data-tauri-drag-region class="flex-grow flex justify-center">
            <ul class="flex gap-4">
                <li>
                    <NuxtLink to="/" class="text-white hover:underline">Home</NuxtLink>
                </li>
                <li>
                    <NuxtLink to="/file-explorer" class="text-white hover:underline">File Explorer</NuxtLink>
                </li>
            </ul>
        </nav>

        <div data-tauri-drag-region class="ml-auto pt-2 mx-4 flex gap-2">
            <div class="w-[30px] h-[30px] shrink-0 border border-white p-0.5 cursor-pointer">
                <UIcon name="i-mdi-window-minimize" class="h-[25px] w-[25px]" />
            </div>
            <div class="w-[30px] h-[30px] shrink-0 border border-white p-0.5 cursor-pointer">
                <UIcon name="i-mdi-window-close" class="h-[25px] w-[25px]" />
            </div>
        </div>
    </header>
</template>