<template>
  <div class="space-y-4">
    <!-- Image Display -->
    <div class="relative group">
      <img 
        :src="imageData" 
        :alt="alt"
        class="max-w-full h-auto rounded-lg border border-gray-200 shadow-sm"
        :class="{ 'cursor-zoom-in': !isZoomed, 'cursor-zoom-out': isZoomed }"
        @click="toggleZoom"
      />
      
      <!-- Overlay Controls -->
      <div class="absolute top-2 right-2 flex space-x-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
        <BaseButton
          v-if="showZoomControl"
          @click="toggleZoom"
          variant="outline"
          size="sm"
          class="bg-white/90 backdrop-blur-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path 
              v-if="!isZoomed"
              stroke-linecap="round" 
              stroke-linejoin="round" 
              stroke-width="2" 
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" 
            />
            <path 
              v-else
              stroke-linecap="round" 
              stroke-linejoin="round" 
              stroke-width="2" 
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM13 10H7" 
            />
          </svg>
        </BaseButton>
        
        <BaseButton
          v-if="showClearControl"
          @click="$emit('clear')"
          variant="outline"
          size="sm"
          class="bg-white/90 backdrop-blur-sm text-red-600 hover:text-red-700"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </BaseButton>
      </div>
      
      <!-- Loading Overlay -->
      <div 
        v-if="isProcessing" 
        class="absolute inset-0 bg-black/50 rounded-lg flex items-center justify-center"
      >
        <div class="bg-white rounded-lg p-4 flex items-center space-x-3">
          <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-primary-600"></div>
          <span class="text-gray-700">{{ processingText }}</span>
        </div>
      </div>
    </div>

    <!-- Image Information -->
    <div v-if="imageInfo && showImageInfo" class="bg-gray-50 rounded-lg p-4">
      <h4 class="font-medium text-gray-900 mb-3">图像信息</h4>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
        <div>
          <span class="text-gray-600">尺寸:</span>
          <div class="font-medium">{{ imageInfo.width }} × {{ imageInfo.height }}</div>
        </div>
        <div>
          <span class="text-gray-600">大小:</span>
          <div class="font-medium">{{ formatFileSize(imageInfo.size) }}</div>
        </div>
        <div v-if="imageInfo.format">
          <span class="text-gray-600">格式:</span>
          <div class="font-medium uppercase">{{ imageInfo.format }}</div>
        </div>
        <div v-if="imageInfo.is_suitable !== undefined">
          <span class="text-gray-600">适用性:</span>
          <div class="font-medium" :class="imageInfo.is_suitable ? 'text-green-600' : 'text-red-600'">
            {{ imageInfo.is_suitable ? '适合' : '不适合' }}
          </div>
        </div>
      </div>
      
      <!-- Quality Indicators -->
      <div v-if="imageInfo.quality_score !== undefined" class="mt-3 pt-3 border-t border-gray-200">
        <div class="flex items-center justify-between">
          <span class="text-gray-600">质量评分:</span>
          <div class="flex items-center space-x-2">
            <div class="w-24 bg-gray-200 rounded-full h-2">
              <div 
                class="h-2 rounded-full transition-all duration-300"
                :class="getQualityColorClass(imageInfo.quality_score)"
                :style="{ width: `${imageInfo.quality_score * 100}%` }"
              ></div>
            </div>
            <span class="text-sm font-medium">{{ Math.round(imageInfo.quality_score * 100) }}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Detection Results -->
    <div v-if="detectionResult && showDetectionResult" class="bg-blue-50 rounded-lg p-4">
      <h4 class="font-medium text-gray-900 mb-3">检测结果</h4>
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-gray-600">输入类型:</span>
          <span class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full text-sm font-medium">
            {{ getInputTypeLabel(detectionResult.inputType) }}
          </span>
        </div>
        
        <div v-if="detectionResult.confidence !== undefined" class="flex items-center justify-between">
          <span class="text-gray-600">置信度:</span>
          <div class="flex items-center space-x-2">
            <div class="w-20 bg-gray-200 rounded-full h-2">
              <div 
                class="bg-blue-500 h-2 rounded-full transition-all duration-300"
                :style="{ width: `${detectionResult.confidence * 100}%` }"
              ></div>
            </div>
            <span class="text-sm font-medium">{{ Math.round(detectionResult.confidence * 100) }}%</span>
          </div>
        </div>
        
        <div v-if="detectionResult.layout" class="pt-2 border-t border-blue-200">
          <span class="text-gray-600 text-sm">布局分析:</span>
          <div class="mt-2 grid grid-cols-2 gap-3 text-sm">
            <div class="flex justify-between">
              <span>公式区域:</span>
              <span class="font-medium">{{ detectionResult.layout.formula_regions.length }}</span>
            </div>
            <div class="flex justify-between">
              <span>文本区域:</span>
              <span class="font-medium">{{ detectionResult.layout.text_regions.length }}</span>
            </div>
            <div class="flex justify-between">
              <span>多公式:</span>
              <span class="font-medium">{{ detectionResult.layout.has_multiple_formulas ? '是' : '否' }}</span>
            </div>
            <div class="flex justify-between">
              <span>包含文本:</span>
              <span class="font-medium">{{ detectionResult.layout.has_text_content ? '是' : '否' }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Action Buttons -->
    <div v-if="showActions" class="flex flex-wrap gap-3">
      <BaseButton
        v-if="showRecognizeButton"
        @click="$emit('recognize')"
        :disabled="isProcessing"
        class="flex items-center space-x-2"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
        <span>开始识别</span>
      </BaseButton>
      
      <BaseButton
        v-if="showAnalyzeButton"
        @click="$emit('analyze')"
        :disabled="isProcessing"
        variant="outline"
        class="flex items-center space-x-2"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
        <span>分析布局</span>
      </BaseButton>
      
      <BaseButton
        v-if="showRetryButton"
        @click="$emit('retry')"
        :disabled="isProcessing"
        variant="outline"
        class="flex items-center space-x-2"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        <span>重试</span>
      </BaseButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import BaseButton from '@/components/BaseButton.vue'
import { InputType, type ImageLayout } from '@/types'

interface ImageInfo {
  width: number
  height: number
  size: number
  format?: string
  is_suitable?: boolean
  quality_score?: number
}

interface DetectionResult {
  inputType: InputType
  confidence?: number
  layout?: ImageLayout
}

interface Props {
  imageData: string
  alt?: string
  imageInfo?: ImageInfo
  detectionResult?: DetectionResult
  isProcessing?: boolean
  processingText?: string
  showImageInfo?: boolean
  showDetectionResult?: boolean
  showZoomControl?: boolean
  showClearControl?: boolean
  showActions?: boolean
  showRecognizeButton?: boolean
  showAnalyzeButton?: boolean
  showRetryButton?: boolean
}

withDefaults(defineProps<Props>(), {
  alt: 'Preview image',
  isProcessing: false,
  processingText: '处理中...',
  showImageInfo: true,
  showDetectionResult: true,
  showZoomControl: true,
  showClearControl: true,
  showActions: true,
  showRecognizeButton: true,
  showAnalyzeButton: false,
  showRetryButton: false
})

defineEmits<{
  clear: []
  recognize: []
  analyze: []
  retry: []
}>()

const isZoomed = ref(false)

function toggleZoom() {
  isZoomed.value = !isZoomed.value
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

function getQualityColorClass(score: number): string {
  if (score >= 0.8) return 'bg-green-500'
  if (score >= 0.6) return 'bg-yellow-500'
  return 'bg-red-500'
}

function getInputTypeLabel(inputType: InputType): string {
  switch (inputType) {
    case InputType.SingleFormula:
      return '单个公式'
    case InputType.Document:
      return '文档'
    default:
      return '未知'
  }
}
</script>