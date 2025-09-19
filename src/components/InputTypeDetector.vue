<template>
  <BaseCard>
    <template #header>
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900">输入类型检测</h3>
        <div v-if="isDetecting" class="flex items-center space-x-2 text-sm text-gray-600">
          <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-600"></div>
          <span>检测中...</span>
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Detection Status -->
      <div v-if="!detectionResult && !isDetecting" class="text-center py-8">
        <div class="w-16 h-16 mx-auto mb-4 text-gray-300">
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" class="w-full h-full">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
              d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
          </svg>
        </div>
        <p class="text-gray-500">请先上传图像进行类型检测</p>
      </div>

      <!-- Automatic Detection Results -->
      <div v-if="detectionResult" class="space-y-4">
        <div class="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-4 border border-blue-200">
          <div class="flex items-center justify-between mb-3">
            <h4 class="font-medium text-gray-900">自动检测结果</h4>
            <div class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              <span class="text-sm text-green-700 font-medium">已完成</span>
            </div>
          </div>
          
          <div class="space-y-3">
            <!-- Detected Type -->
            <div class="flex items-center justify-between">
              <span class="text-gray-700">检测类型:</span>
              <div class="flex items-center space-x-2">
                <div class="w-6 h-6 text-blue-600">
                  <svg v-if="detectionResult.inputType === InputType.SingleFormula" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 7h6m0 10v-3m-3+3.5a2 2 0 01-4 0M9 17h6m-3-5v4" />
                  </svg>
                  <svg v-else fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                </div>
                <span class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full text-sm font-medium">
                  {{ getInputTypeLabel(detectionResult.inputType) }}
                </span>
              </div>
            </div>

            <!-- Confidence Score -->
            <div v-if="detectionResult.confidence !== undefined" class="space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-gray-700">置信度:</span>
                <span class="text-sm font-medium">{{ Math.round(detectionResult.confidence * 100) }}%</span>
              </div>
              
              <!-- Confidence Bar -->
              <div class="w-full bg-gray-200 rounded-full h-3 overflow-hidden">
                <div 
                  class="h-full rounded-full transition-all duration-500 ease-out"
                  :class="getConfidenceColorClass(detectionResult.confidence)"
                  :style="{ width: `${detectionResult.confidence * 100}%` }"
                ></div>
              </div>
              
              <!-- Confidence Level Indicator -->
              <div class="flex justify-between text-xs text-gray-500">
                <span>低</span>
                <span>中</span>
                <span>高</span>
              </div>
            </div>

            <!-- Detection Details -->
            <div v-if="detectionResult.layout" class="pt-3 border-t border-blue-200">
              <h5 class="text-sm font-medium text-gray-700 mb-2">布局分析详情</h5>
              <div class="grid grid-cols-2 gap-3 text-sm">
                <div class="bg-white rounded-lg p-3 border">
                  <div class="flex items-center justify-between">
                    <span class="text-gray-600">公式区域</span>
                    <span class="font-semibold text-blue-600">{{ detectionResult.layout.formula_regions.length }}</span>
                  </div>
                </div>
                <div class="bg-white rounded-lg p-3 border">
                  <div class="flex items-center justify-between">
                    <span class="text-gray-600">文本区域</span>
                    <span class="font-semibold text-green-600">{{ detectionResult.layout.text_regions.length }}</span>
                  </div>
                </div>
                <div class="bg-white rounded-lg p-3 border">
                  <div class="flex items-center justify-between">
                    <span class="text-gray-600">多公式</span>
                    <span :class="detectionResult.layout.has_multiple_formulas ? 'text-orange-600' : 'text-gray-500'">
                      {{ detectionResult.layout.has_multiple_formulas ? '是' : '否' }}
                    </span>
                  </div>
                </div>
                <div class="bg-white rounded-lg p-3 border">
                  <div class="flex items-center justify-between">
                    <span class="text-gray-600">包含文本</span>
                    <span :class="detectionResult.layout.has_text_content ? 'text-purple-600' : 'text-gray-500'">
                      {{ detectionResult.layout.has_text_content ? '是' : '否' }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Manual Override Section -->
        <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
          <div class="flex items-center justify-between mb-3">
            <h4 class="font-medium text-gray-900">手动调整</h4>
            <BaseButton
              v-if="hasManualOverride"
              @click="resetToAutomatic"
              variant="outline"
              size="sm"
              class="text-xs"
            >
              恢复自动检测
            </BaseButton>
          </div>
          
          <div class="space-y-3">
            <p class="text-sm text-gray-600">
              如果自动检测结果不准确，您可以手动选择输入类型：
            </p>
            
            <!-- Type Selection Buttons -->
            <div class="grid grid-cols-2 gap-3">
              <button
                @click="setManualType(InputType.SingleFormula)"
                :class="[
                  'p-4 rounded-lg border-2 transition-all duration-200 text-left',
                  currentInputType === InputType.SingleFormula
                    ? 'border-blue-500 bg-blue-50 text-blue-900'
                    : 'border-gray-200 bg-white hover:border-gray-300 text-gray-700'
                ]"
              >
                <div class="flex items-center space-x-3">
                  <div class="w-8 h-8 text-blue-600">
                    <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 7h6m0 10v-3m-3+3.5a2 2 0 01-4 0M9 17h6m-3-5v4" />
                    </svg>
                  </div>
                  <div>
                    <div class="font-medium">单个公式</div>
                    <div class="text-sm text-gray-500">单独的数学公式或表达式</div>
                  </div>
                </div>
              </button>
              
              <button
                @click="setManualType(InputType.Document)"
                :class="[
                  'p-4 rounded-lg border-2 transition-all duration-200 text-left',
                  currentInputType === InputType.Document
                    ? 'border-green-500 bg-green-50 text-green-900'
                    : 'border-gray-200 bg-white hover:border-gray-300 text-gray-700'
                ]"
              >
                <div class="flex items-center space-x-3">
                  <div class="w-8 h-8 text-green-600">
                    <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                  </div>
                  <div>
                    <div class="font-medium">文档</div>
                    <div class="text-sm text-gray-500">包含多个公式和文本的文档</div>
                  </div>
                </div>
              </button>
            </div>
            
            <!-- Manual Override Indicator -->
            <div v-if="hasManualOverride" class="flex items-center space-x-2 text-sm text-orange-600 bg-orange-50 rounded-lg p-3">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
              <span>已手动设置为 {{ currentInputType ? getInputTypeLabel(currentInputType) : '未知' }}</span>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex justify-between items-center pt-4 border-t border-gray-200">
          <BaseButton
            @click="redetect"
            variant="outline"
            :disabled="isDetecting"
            class="flex items-center space-x-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span>重新检测</span>
          </BaseButton>
          
          <BaseButton
            @click="confirmType"
            :disabled="!currentInputType"
            class="flex items-center space-x-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            <span>确认类型</span>
          </BaseButton>
        </div>
      </div>

      <!-- Error Display -->
      <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
        <div class="flex items-center space-x-2">
          <svg class="w-5 h-5 text-red-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div>
            <p class="text-red-700 font-medium">检测失败</p>
            <p class="text-red-600 text-sm">{{ error }}</p>
          </div>
        </div>
      </div>
    </div>
  </BaseCard>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import BaseButton from '@/components/BaseButton.vue'
import BaseCard from '@/components/BaseCard.vue'
import { InputType, type ImageLayout } from '@/types'

interface DetectionResult {
  inputType: InputType
  confidence?: number
  layout?: ImageLayout
}

interface Props {
  detectionResult?: DetectionResult
  isDetecting?: boolean
  error?: string
}

const props = withDefaults(defineProps<Props>(), {
  isDetecting: false
})

const emit = defineEmits<{
  typeChanged: [inputType: InputType, isManual: boolean]
  redetect: []
  confirm: [inputType: InputType, isManual: boolean]
}>()

const manualInputType = ref<InputType | null>(null)

const currentInputType = computed(() => {
  return manualInputType.value || props.detectionResult?.inputType || null
})

const hasManualOverride = computed(() => {
  return manualInputType.value !== null && 
         manualInputType.value !== props.detectionResult?.inputType
})

// Watch for detection result changes and reset manual override if it matches
watch(() => props.detectionResult?.inputType, (newType) => {
  if (newType && manualInputType.value === newType) {
    manualInputType.value = null
  }
})

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

function getConfidenceColorClass(confidence: number): string {
  if (confidence >= 0.8) return 'bg-green-500'
  if (confidence >= 0.6) return 'bg-yellow-500'
  if (confidence >= 0.4) return 'bg-orange-500'
  return 'bg-red-500'
}

function setManualType(inputType: InputType) {
  manualInputType.value = inputType
  emit('typeChanged', inputType, true)
}

function resetToAutomatic() {
  manualInputType.value = null
  if (props.detectionResult?.inputType) {
    emit('typeChanged', props.detectionResult.inputType, false)
  }
}

function redetect() {
  manualInputType.value = null
  emit('redetect')
}

function confirmType() {
  if (currentInputType.value) {
    emit('confirm', currentInputType.value, hasManualOverride.value)
  }
}
</script>