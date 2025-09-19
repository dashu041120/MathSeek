<template>
  <div class="space-y-6">
    <!-- Image Input Section -->
    <ImageInput
      @image-selected="handleImageSelected"
    />

    <!-- Input Type Detection Section -->
    <InputTypeDetector
      v-if="currentImage"
      :detection-result="detectionResult || undefined"
      :is-detecting="isDetecting"
      :error="detectionError || undefined"
      @type-changed="handleTypeChanged"
      @redetect="handleRedetect"
      @confirm="handleConfirm"
    />

    <!-- Next Steps -->
    <BaseCard v-if="confirmedType">
      <template #header>
        <h3 class="text-lg font-semibold text-gray-900">下一步操作</h3>
      </template>
      
      <div class="space-y-4">
        <p class="text-gray-600">
          输入类型已确认为 <span class="font-medium text-blue-600">{{ getInputTypeLabel(confirmedType) }}</span>，
          您可以继续进行公式识别。
        </p>
        
        <div class="flex space-x-4">
          <BaseButton
            @click="startRecognition"
            :disabled="isProcessing"
            class="flex items-center space-x-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
            <span>开始识别</span>
          </BaseButton>
          
          <BaseButton
            @click="resetWorkflow"
            variant="outline"
            class="flex items-center space-x-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span>重新开始</span>
          </BaseButton>
        </div>
      </div>
    </BaseCard>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ImageInput from '@/components/ImageInput.vue'
import InputTypeDetector from '@/components/InputTypeDetector.vue'
import BaseCard from '@/components/BaseCard.vue'
import BaseButton from '@/components/BaseButton.vue'
import { useTauri } from '@/composables/useTauri'
import { InputType, type ImageLayout } from '@/types'

const currentImage = ref<string | null>(null)
const currentLayout = ref<ImageLayout | null>(null)
const detectionResult = ref<{
  inputType: InputType
  confidence?: number
  layout?: ImageLayout
} | null>(null)
const isDetecting = ref(false)
const detectionError = ref<string | null>(null)
const confirmedType = ref<InputType | null>(null)
const isProcessing = ref(false)

const { detectInputType, analyzeImageLayout, getDetectionConfidence } = useTauri()

async function handleImageSelected(imageData: string, inputType: string, layout: ImageLayout) {
  currentImage.value = imageData
  currentLayout.value = layout
  
  // Set initial detection result
  detectionResult.value = {
    inputType: inputType as InputType,
    layout: layout
  }
  
  // Get confidence score
  try {
    const confidence = await getDetectionConfidence(imageData)
    if (detectionResult.value) {
      detectionResult.value.confidence = confidence
    }
  } catch (error) {
    console.error('Failed to get detection confidence:', error)
  }
}

function handleTypeChanged(inputType: InputType, _isManual: boolean) {
  if (detectionResult.value) {
    detectionResult.value = {
      ...detectionResult.value,
      inputType
    }
  }
  
  // Reset confirmation when type changes
  confirmedType.value = null
}

async function handleRedetect() {
  if (!currentImage.value) return
  
  isDetecting.value = true
  detectionError.value = null
  
  try {
    // Re-detect input type
    const inputType = await detectInputType(currentImage.value)
    
    // Re-analyze layout
    const layout = await analyzeImageLayout(currentImage.value)
    
    // Get confidence
    const confidence = await getDetectionConfidence(currentImage.value)
    
    detectionResult.value = {
      inputType: inputType as InputType,
      confidence,
      layout
    }
    
    currentLayout.value = layout
    confirmedType.value = null
    
  } catch (error) {
    detectionError.value = error instanceof Error ? error.message : '重新检测失败'
  } finally {
    isDetecting.value = false
  }
}

function handleConfirm(inputType: InputType, _isManual: boolean) {
  confirmedType.value = inputType
}

function startRecognition() {
  if (!currentImage.value || !confirmedType.value) return
  
  isProcessing.value = true
  
  // TODO: Implement formula recognition
  // This will be implemented in later tasks
  console.log('Starting recognition for type:', confirmedType.value)
  
  setTimeout(() => {
    isProcessing.value = false
    // Placeholder for recognition results
  }, 2000)
}

function resetWorkflow() {
  currentImage.value = null
  currentLayout.value = null
  detectionResult.value = null
  confirmedType.value = null
  detectionError.value = null
  isDetecting.value = false
  isProcessing.value = false
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