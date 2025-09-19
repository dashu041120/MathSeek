<template>
  <button
    :class="[
      'font-medium py-2 px-4 rounded-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2',
      variantClasses,
      sizeClasses,
      { 'opacity-50 cursor-not-allowed': disabled }
    ]"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'primary' | 'secondary' | 'danger' | 'outline'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false
})

defineEmits<{
  click: [event: MouseEvent]
}>()

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'primary':
      return 'bg-primary-600 hover:bg-primary-700 text-white focus:ring-primary-500'
    case 'secondary':
      return 'bg-gray-200 hover:bg-gray-300 text-gray-800 focus:ring-gray-500'
    case 'danger':
      return 'bg-red-600 hover:bg-red-700 text-white focus:ring-red-500'
    case 'outline':
      return 'border border-gray-300 bg-white hover:bg-gray-50 text-gray-700 focus:ring-primary-500'
    default:
      return 'bg-primary-600 hover:bg-primary-700 text-white focus:ring-primary-500'
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'text-sm py-1 px-3'
    case 'md':
      return 'text-base py-2 px-4'
    case 'lg':
      return 'text-lg py-3 px-6'
    default:
      return 'text-base py-2 px-4'
  }
})
</script>