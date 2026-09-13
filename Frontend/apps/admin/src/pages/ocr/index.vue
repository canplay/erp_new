<template>
  <q-page class="q-pa-md">
    <div class="q-mb-md">
      <div class="text-h6">智能识别（OCR）</div>
      <div class="text-caption text-grey-7">
        拍照/上传图片 → PaddleOCR 文字识别；表计模式识别读数（后端内网接入，仅本系统鉴权用户可用）
      </div>
    </div>
    <OcrWorkbench ledger />
  </q-page>
</template>

<script setup lang="ts">
import { OcrWorkbench } from '@erp-new-frontend-monorepo/pages';
</script>
