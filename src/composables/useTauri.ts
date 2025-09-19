import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, SystemStatus, ImageLayout } from '@/types'

export function useTauri() {
  // Get application version
  const getAppVersion = async (): Promise<string> => {
    try {
      return await invoke('get_app_version')
    } catch (error) {
      console.error('Failed to get app version:', error)
      return '0.1.0'
    }
  }

  // Check system status
  const checkSystemStatus = async (): Promise<SystemStatus> => {
    try {
      return await invoke('check_system_status')
    } catch (error) {
      console.error('Failed to check system status:', error)
      return {
        clipboard_available: false,
        screenshot_available: false,
        api_configured: false,
        render_engine_ready: false
      }
    }
  }

  // Save configuration
  const saveConfig = async (config: AppConfig): Promise<void> => {
    try {
      await invoke('save_config', { config })
    } catch (error) {
      console.error('Failed to save config:', error)
      throw error
    }
  }

  // Load configuration
  const loadConfig = async (): Promise<AppConfig | null> => {
    try {
      return await invoke('load_config')
    } catch (error) {
      console.error('Failed to load config:', error)
      return null
    }
  }

  // Image processing functions
  const captureScreenshot = async (): Promise<string> => {
    try {
      return await invoke('capture_screenshot')
    } catch (error) {
      console.error('Failed to capture screenshot:', error)
      throw error
    }
  }

  const getClipboardImage = async (): Promise<string | null> => {
    try {
      return await invoke('get_clipboard_image')
    } catch (error) {
      console.error('Failed to get clipboard image:', error)
      throw error
    }
  }

  const validateImageData = async (base64Data: string): Promise<boolean> => {
    try {
      return await invoke('validate_image_data', { base64Data })
    } catch (error) {
      console.error('Failed to validate image data:', error)
      throw error
    }
  }

  const preprocessImage = async (base64Data: string): Promise<string> => {
    try {
      return await invoke('preprocess_image', { base64Data })
    } catch (error) {
      console.error('Failed to preprocess image:', error)
      throw error
    }
  }

  const getImageInfo = async (base64Data: string): Promise<any> => {
    try {
      return await invoke('get_image_info', { base64Data })
    } catch (error) {
      console.error('Failed to get image info:', error)
      throw error
    }
  }

  const detectInputType = async (base64Data: string): Promise<string> => {
    try {
      return await invoke('detect_input_type', { base64Data })
    } catch (error) {
      console.error('Failed to detect input type:', error)
      throw error
    }
  }

  const analyzeImageLayout = async (base64Data: string): Promise<ImageLayout> => {
    try {
      return await invoke('analyze_image_layout', { base64Data })
    } catch (error) {
      console.error('Failed to analyze image layout:', error)
      throw error
    }
  }

  const getDetectionConfidence = async (base64Data: string): Promise<number> => {
    try {
      return await invoke('get_detection_confidence', { base64Data })
    } catch (error) {
      console.error('Failed to get detection confidence:', error)
      throw error
    }
  }

  return {
    getAppVersion,
    checkSystemStatus,
    saveConfig,
    loadConfig,
    captureScreenshot,
    getClipboardImage,
    validateImageData,
    preprocessImage,
    getImageInfo,
    detectInputType,
    analyzeImageLayout,
    getDetectionConfidence
  }
}