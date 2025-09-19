<script setup lang="ts">
import { ref, computed } from 'vue';
import BaseButton from '@/components/BaseButton.vue';
import BaseCard from '@/components/BaseCard.vue';
import ImagePreview from '@/components/ImagePreview.vue';
import { useTauri } from '@/composables/useTauri';
import type { ImageLayout, InputType } from '@/types';

const emit = defineEmits<{
  imageSelected: [imageData: string, inputType: string, layout: ImageLayout];
}>();

const selectedImage = ref<string | null>(null);
const isProcessing = ref(false);
const error = ref<string | null>(null);
const imageInfo = ref<any>(null);
const detectedType = ref<string | null>(null);
const detectionConfidence = ref<number | null>(null);
const imageLayout = ref<ImageLayout | null>(null);
const isDragOver = ref(false);
const dragCounter = ref(0);

const { 
  captureScreenshot, 
  getClipboardImage, 
  validateImageData, 
  preprocessImage,
  getImageInfo,
  detectInputType,
  analyzeImageLayout,
  getDetectionConfidence
} = useTauri();

const hasImage = computed(() => selectedImage.value !== null);

const detectionResult = computed(() => {
  if (!detectedType.value) return undefined;
  
  return {
    inputType: detectedType.value as InputType,
    confidence: detectionConfidence.value || undefined,
    layout: imageLayout.value || undefined
  };
});

async function handleScreenshot() {
  isProcessing.value = true;
  error.value = null;
  
  try {
    const imageData = await captureScreenshot();
    await processImage(imageData);
  } catch (err) {
    error.value = err instanceof Error ? err.message : '截图失败';
  } finally {
    isProcessing.value = false;
  }
}

async function handleClipboardPaste() {
  isProcessing.value = true;
  error.value = null;
  
  try {
    const imageData = await getClipboardImage();
    if (imageData) {
      await processImage(imageData);
    } else {
      error.value = '剪贴板中没有图像数据';
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : '获取剪贴板图像失败';
  } finally {
    isProcessing.value = false;
  }
}

async function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  
  if (!file) return;
  
  isProcessing.value = true;
  error.value = null;
  
  try {
    const reader = new FileReader();
    reader.onload = async (e) => {
      const result = e.target?.result as string;
      await processImage(result);
    };
    reader.readAsDataURL(file);
  } catch (err) {
    error.value = err instanceof Error ? err.message : '文件上传失败';
    isProcessing.value = false;
  }
}

async function processImage(imageData: string) {
  try {
    // Validate image
    const isValid = await validateImageData(imageData);
    if (!isValid) {
      throw new Error('无效的图像数据');
    }

    // Get image info
    imageInfo.value = await getImageInfo(imageData);
    
    if (!imageInfo.value.is_suitable) {
      throw new Error('图像不适合处理（可能太小、太大或质量不佳）');
    }

    // Preprocess image
    const processedImage = await preprocessImage(imageData);
    selectedImage.value = processedImage;

    // Detect input type
    detectedType.value = await detectInputType(processedImage);
    
    // Get detection confidence
    detectionConfidence.value = await getDetectionConfidence(processedImage);
    
    // Analyze layout
    imageLayout.value = await analyzeImageLayout(processedImage);

    // Emit the processed image data
    emit('imageSelected', processedImage, detectedType.value, imageLayout.value);
    
  } catch (err) {
    error.value = err instanceof Error ? err.message : '图像处理失败';
  } finally {
    isProcessing.value = false;
  }
}

function clearImage() {
  selectedImage.value = null;
  imageInfo.value = null;
  detectedType.value = null;
  detectionConfidence.value = null;
  imageLayout.value = null;
  error.value = null;
}

// Drag and drop handlers
function handleDragEnter(event: DragEvent) {
  event.preventDefault();
  dragCounter.value++;
  isDragOver.value = true;
}

function handleDragLeave(event: DragEvent) {
  event.preventDefault();
  dragCounter.value--;
  if (dragCounter.value === 0) {
    isDragOver.value = false;
  }
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
}

async function handleDrop(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = false;
  dragCounter.value = 0;
  
  const files = event.dataTransfer?.files;
  if (!files || files.length === 0) return;
  
  const file = files[0];
  if (!file.type.startsWith('image/')) {
    error.value = '请拖拽图像文件';
    return;
  }
  
  isProcessing.value = true;
  error.value = null;
  
  try {
    const reader = new FileReader();
    reader.onload = async (e) => {
      const result = e.target?.result as string;
      await processImage(result);
    };
    reader.readAsDataURL(file);
  } catch (err) {
    error.value = err instanceof Error ? err.message : '文件处理失败';
    isProcessing.value = false;
  }
}

function handleRecognize() {
  if (selectedImage.value && detectedType.value && imageLayout.value) {
    emit('imageSelected', selectedImage.value, detectedType.value, imageLayout.value);
  }
}

function handleAnalyze() {
  // Re-analyze the current image
  if (selectedImage.value) {
    processImage(selectedImage.value);
  }
}
</script>

<template>
  <BaseCard>
    <template #header>
      <h3 class="text-lg font-semibold text-gray-900">图像输入</h3>
    </template>

    <div class="space-y-4">
      <!-- Input Methods -->
      <div v-if="!hasImage" class="space-y-4">
        <!-- Drag and Drop Zone -->
        <div 
          class="relative border-2 border-dashed rounded-lg p-8 text-center transition-colors duration-200"
          :class="[
            isDragOver 
              ? 'border-primary-400 bg-primary-50' 
              : 'border-gray-300 hover:border-gray-400'
          ]"
          @dragenter="handleDragEnter"
          @dragleave="handleDragLeave"
          @dragover="handleDragOver"
          @drop="handleDrop"
        >
          <div class="space-y-4">
            <div class="mx-auto w-16 h-16 text-gray-400">
              <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" class="w-full h-full">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
              </svg>
            </div>
            <div>
              <p class="text-lg font-medium text-gray-900">
                {{ isDragOver ? '释放以上传图像' : '拖拽图像到此处' }}
              </p>
              <p class="text-gray-500 mt-1">或使用下方的输入方式</p>
            </div>
          </div>
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <BaseButton 
            @click="handleScreenshot" 
            :disabled="isProcessing"
            class="flex flex-col items-center space-y-2 p-4 h-auto"
          >
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            <span>截图</span>
          </BaseButton>

          <BaseButton 
            @click="handleClipboardPaste" 
            :disabled="isProcessing"
            variant="outline"
            class="flex flex-col items-center space-y-2 p-4 h-auto"
          >
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
            </svg>
            <span>粘贴</span>
          </BaseButton>

          <label class="cursor-pointer">
            <BaseButton 
              as="div"
              variant="outline"
              :disabled="isProcessing"
              class="flex flex-col items-center space-y-2 p-4 h-auto w-full"
            >
              <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
              </svg>
              <span>上传文件</span>
            </BaseButton>
            <input 
              type="file" 
              accept="image/*" 
              @change="handleFileUpload"
              class="hidden"
              :disabled="isProcessing"
            />
          </label>
        </div>

        <!-- Processing indicator -->
        <div v-if="isProcessing" class="text-center py-4">
          <div class="inline-flex items-center space-x-2">
            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-600"></div>
            <span class="text-gray-600">处理中...</span>
          </div>
        </div>
      </div>

      <!-- Image Preview and Analysis -->
      <div v-if="hasImage">
        <ImagePreview
          :image-data="selectedImage || ''"
          :image-info="imageInfo"
          :detection-result="detectionResult"
          :is-processing="isProcessing"
          processing-text="分析图像中..."
          @clear="clearImage"
          @recognize="handleRecognize"
          @analyze="handleAnalyze"
        />
      </div>

      <!-- Error Display -->
      <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
        <div class="flex items-center space-x-2">
          <svg class="w-5 h-5 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-red-700">{{ error }}</span>
        </div>
      </div>
    </div>
  </BaseCard>
</template>