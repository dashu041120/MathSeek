<template>
  <div
    :class="[
      'bg-white rounded-lg shadow-md border border-gray-200',
      paddingClasses,
      { 'hover:shadow-lg transition-shadow duration-200': hoverable }
    ]"
  >
    <div v-if="$slots.header" class="border-b border-gray-200 pb-4 mb-4">
      <slot name="header" />
    </div>
    
    <div class="flex-1">
      <slot />
    </div>
    
    <div v-if="$slots.footer" class="border-t border-gray-200 pt-4 mt-4">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  padding?: 'none' | 'sm' | 'md' | 'lg'
  hoverable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  padding: 'md',
  hoverable: false
})

const paddingClasses = computed(() => {
  switch (props.padding) {
    case 'none':
      return ''
    case 'sm':
      return 'p-4'
    case 'md':
      return 'p-6'
    case 'lg':
      return 'p-8'
    default:
      return 'p-6'
  }
})
</script>