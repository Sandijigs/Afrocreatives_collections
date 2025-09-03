import { useState, useCallback } from 'react'

export interface IPFSFile {
  name: string
  hash: string
  size: number
  type: string
  url: string
  uploadedAt: number
}

export interface ProjectMetadata {
  title: string
  description: string
  culturalElements: string[]
  region: string
  creator: string
  images: IPFSFile[]
  videos: IPFSFile[]
  audio: IPFSFile[]
  documents: IPFSFile[]
  tags: string[]
  category: string
}

export function useIPFS() {
  const [uploadProgress, setUploadProgress] = useState<{ [key: string]: number }>({})
  const [isUploading, setIsUploading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [uploadedFiles, setUploadedFiles] = useState<IPFSFile[]>([])

  // Mock IPFS upload
  const uploadFile = useCallback(async (
    file: File,
    onProgress?: (progress: number) => void
  ): Promise<IPFSFile> => {
    setIsUploading(true)
    setError(null)
    
    try {
      const fileId = `${file.name}-${Date.now()}`
      
      // Simulate upload progress
      for (let progress = 0; progress <= 100; progress += 10) {
        await new Promise(resolve => setTimeout(resolve, 100))
        setUploadProgress(prev => ({ ...prev, [fileId]: progress }))
        onProgress?.(progress)
      }
      
      // Generate mock IPFS hash
      const hash = `Qm${Math.random().toString(36).substr(2, 44)}`
      
      const ipfsFile: IPFSFile = {
        name: file.name,
        hash,
        size: file.size,
        type: file.type,
        url: `https://ipfs.io/ipfs/${hash}`,
        uploadedAt: Date.now(),
      }
      
      setUploadedFiles(prev => [...prev, ipfsFile])
      
      // Clean up progress tracking
      setUploadProgress(prev => {
        const updated = { ...prev }
        delete updated[fileId]
        return updated
      })
      
      return ipfsFile
    } catch (err) {
      setError('Failed to upload file to IPFS')
      throw err
    } finally {
      setIsUploading(false)
    }
  }, [])

  // Upload multiple files
  const uploadFiles = useCallback(async (
    files: File[],
    onProgress?: (fileIndex: number, progress: number) => void
  ): Promise<IPFSFile[]> => {
    const results: IPFSFile[] = []
    
    for (let i = 0; i < files.length; i++) {
      const file = files[i]
      try {
        const result = await uploadFile(file, (progress) => {
          onProgress?.(i, progress)
        })
        results.push(result)
      } catch (err) {
        console.error(`Failed to upload file ${file.name}:`, err)
      }
    }
    
    return results
  }, [uploadFile])

  // Upload project metadata as JSON
  const uploadMetadata = useCallback(async (metadata: ProjectMetadata): Promise<IPFSFile> => {
    setIsUploading(true)
    setError(null)
    
    try {
      // Convert metadata to JSON file
      const jsonContent = JSON.stringify(metadata, null, 2)
      const jsonFile = new File([jsonContent], 'metadata.json', {
        type: 'application/json'
      })
      
      const result = await uploadFile(jsonFile)
      return result
    } catch (err) {
      setError('Failed to upload metadata')
      throw err
    } finally {
      setIsUploading(false)
    }
  }, [uploadFile])

  // Retrieve file from IPFS
  const getFile = useCallback(async (hash: string): Promise<string> => {
    try {
      // In a real implementation, this would fetch from IPFS
      // For mock purposes, return the constructed URL
      return `https://ipfs.io/ipfs/${hash}`
    } catch (err) {
      setError('Failed to retrieve file from IPFS')
      throw err
    }
  }, [])

  // Pin file to IPFS (ensure persistence)
  const pinFile = useCallback(async (hash: string): Promise<boolean> => {
    setIsUploading(true)
    setError(null)
    
    try {
      // Simulate pinning process
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Mock successful pinning
      return true
    } catch (err) {
      setError('Failed to pin file')
      return false
    } finally {
      setIsUploading(false)
    }
  }, [])

  // Get file metadata
  const getFileMetadata = useCallback(async (hash: string) => {
    try {
      // Simulate metadata retrieval
      await new Promise(resolve => setTimeout(resolve, 500))
      
      return {
        hash,
        size: Math.floor(Math.random() * 10000000), // Random size
        pinned: true,
        providers: Math.floor(Math.random() * 50) + 10, // 10-60 providers
      }
    } catch (err) {
      setError('Failed to get file metadata')
      throw err
    }
  }, [])

  // Validate file type for cultural content
  const validateCulturalContent = useCallback((file: File): boolean => {
    const allowedTypes = [
      // Images
      'image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml',
      // Video
      'video/mp4', 'video/webm', 'video/quicktime',
      // Audio
      'audio/mpeg', 'audio/wav', 'audio/ogg', 'audio/aac',
      // Documents
      'application/pdf', 'text/plain', 'application/json',
    ]
    
    const maxSize = 100 * 1024 * 1024 // 100MB
    
    if (!allowedTypes.includes(file.type)) {
      setError(`File type ${file.type} is not supported`)
      return false
    }
    
    if (file.size > maxSize) {
      setError('File size exceeds 100MB limit')
      return false
    }
    
    return true
  }, [])

  // Generate IPFS URL
  const generateIPFSUrl = useCallback((hash: string, gateway?: string): string => {
    const defaultGateway = 'https://ipfs.io/ipfs/'
    const selectedGateway = gateway || defaultGateway
    return `${selectedGateway}${hash}`
  }, [])

  // Get popular IPFS gateways
  const getGateways = useCallback((): string[] => {
    return [
      'https://ipfs.io/ipfs/',
      'https://gateway.pinata.cloud/ipfs/',
      'https://cloudflare-ipfs.com/ipfs/',
      'https://dweb.link/ipfs/',
    ]
  }, [])

  // Calculate storage cost estimate
  const estimateStorageCost = useCallback((totalSize: number): number => {
    // Mock pricing: $0.01 per MB per month
    const costPerMBPerMonth = 0.01
    const sizeInMB = totalSize / (1024 * 1024)
    return Math.ceil(sizeInMB * costPerMBPerMonth * 100) / 100 // Round to 2 decimals
  }, [])

  // Clear uploaded files
  const clearUploadedFiles = useCallback(() => {
    setUploadedFiles([])
    setUploadProgress({})
    setError(null)
  }, [])

  return {
    uploadProgress,
    isUploading,
    error,
    uploadedFiles,
    uploadFile,
    uploadFiles,
    uploadMetadata,
    getFile,
    pinFile,
    getFileMetadata,
    validateCulturalContent,
    generateIPFSUrl,
    getGateways,
    estimateStorageCost,
    clearUploadedFiles,
  }
}