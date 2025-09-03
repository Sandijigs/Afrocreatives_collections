import { useState, useEffect } from 'react'

export interface ENSProfile {
  domain: string
  address: string
  name?: string
  bio?: string
  avatar?: string
  culturalBackground?: string
  specialties?: string[]
  socialLinks?: {
    twitter?: string
    instagram?: string
    website?: string
  }
  isVerified: boolean
  createdAt: number
}

export function useENS() {
  const [profile, setProfile] = useState<ENSProfile | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Mock ENS resolution
  const resolveDomain = async (domain: string): Promise<ENSProfile | null> => {
    setIsLoading(true)
    setError(null)
    
    try {
      // Simulate ENS resolution
      await new Promise(resolve => setTimeout(resolve, 800))
      
      // Mock profiles for demonstration
      const mockProfiles: { [key: string]: ENSProfile } = {
        'kemi.afro': {
          domain: 'kemi.afro',
          address: '0x1234567890abcdef',
          name: 'Kemi Adebayo',
          bio: 'Afrobeats producer from Lagos, creating authentic Nigerian sounds',
          avatar: '/avatars/kemi.jpg',
          culturalBackground: 'Yoruba, Nigeria',
          specialties: ['Music Production', 'Afrobeats', 'Sound Engineering'],
          socialLinks: {
            twitter: '@kemiadebayo',
            instagram: '@kemi_beats',
            website: 'https://kemibeats.com',
          },
          isVerified: true,
          createdAt: Date.now() - 86400000 * 30, // 30 days ago
        },
        'jelani.afro': {
          domain: 'jelani.afro',
          address: '0xabcdef1234567890',
          name: 'Jelani Mwangi',
          bio: 'Digital artist merging Maasai traditions with futuristic concepts',
          avatar: '/avatars/jelani.jpg',
          culturalBackground: 'Maasai, Kenya',
          specialties: ['Digital Art', 'NFT Creation', 'Cultural Design'],
          socialLinks: {
            twitter: '@jelani_art',
            website: 'https://jelaniart.com',
          },
          isVerified: true,
          createdAt: Date.now() - 86400000 * 45, // 45 days ago
        },
        'amara.afro': {
          domain: 'amara.afro',
          address: '0x9876543210fedcba',
          name: 'Amara Okonkwo',
          bio: 'Film director documenting Ghanaian cultural heritage',
          avatar: '/avatars/amara.jpg',
          culturalBackground: 'Akan, Ghana',
          specialties: ['Film Direction', 'Documentary', 'Cultural Storytelling'],
          socialLinks: {
            twitter: '@amara_films',
            instagram: '@amarastories',
          },
          isVerified: true,
          createdAt: Date.now() - 86400000 * 60, // 60 days ago
        },
      }
      
      const resolvedProfile = mockProfiles[domain.toLowerCase()]
      setProfile(resolvedProfile || null)
      return resolvedProfile || null
      
    } catch (err) {
      setError('Failed to resolve ENS domain')
      return null
    } finally {
      setIsLoading(false)
    }
  }

  // Check domain availability
  const checkAvailability = async (domain: string): Promise<boolean> => {
    setIsLoading(true)
    setError(null)
    
    try {
      // Simulate availability check
      await new Promise(resolve => setTimeout(resolve, 500))
      
      // Mock some unavailable domains
      const unavailableDomains = ['kemi.afro', 'jelani.afro', 'amara.afro', 'test.afro']
      const isAvailable = !unavailableDomains.includes(domain.toLowerCase())
      
      return isAvailable
    } catch (err) {
      setError('Failed to check domain availability')
      return false
    } finally {
      setIsLoading(false)
    }
  }

  // Register a new .afro domain
  const registerDomain = async (domain: string, profileData: Partial<ENSProfile>) => {
    setIsLoading(true)
    setError(null)
    
    try {
      // Simulate domain registration
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      const newProfile: ENSProfile = {
        domain: domain.toLowerCase(),
        address: `0x${Math.random().toString(16).substr(2, 16)}`,
        name: profileData.name || '',
        bio: profileData.bio || '',
        avatar: profileData.avatar,
        culturalBackground: profileData.culturalBackground,
        specialties: profileData.specialties || [],
        socialLinks: profileData.socialLinks || {},
        isVerified: false, // Requires verification
        createdAt: Date.now(),
      }
      
      setProfile(newProfile)
      return newProfile
    } catch (err) {
      setError('Failed to register domain')
      throw err
    } finally {
      setIsLoading(false)
    }
  }

  // Update profile
  const updateProfile = async (updates: Partial<ENSProfile>) => {
    if (!profile) {
      setError('No profile to update')
      return
    }
    
    setIsLoading(true)
    setError(null)
    
    try {
      // Simulate profile update
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      const updatedProfile = { ...profile, ...updates }
      setProfile(updatedProfile)
      return updatedProfile
    } catch (err) {
      setError('Failed to update profile')
      throw err
    } finally {
      setIsLoading(false)
    }
  }

  // Get domain price
  const getDomainPrice = async (domain: string): Promise<number> => {
    try {
      // Mock pricing logic
      const length = domain.replace('.afro', '').length
      let basePrice = 50 // Base price in USD
      
      if (length <= 3) basePrice = 200
      else if (length <= 5) basePrice = 100
      else if (length <= 8) basePrice = 75
      
      return basePrice
    } catch (err) {
      setError('Failed to get domain price')
      return 0
    }
  }

  // Search domains
  const searchDomains = async (query: string): Promise<ENSProfile[]> => {
    setIsLoading(true)
    setError(null)
    
    try {
      // Simulate search
      await new Promise(resolve => setTimeout(resolve, 600))
      
      // Mock search results
      const allProfiles = [
        await resolveDomain('kemi.afro'),
        await resolveDomain('jelani.afro'),
        await resolveDomain('amara.afro'),
      ].filter(Boolean) as ENSProfile[]
      
      const results = allProfiles.filter(profile =>
        profile.name?.toLowerCase().includes(query.toLowerCase()) ||
        profile.bio?.toLowerCase().includes(query.toLowerCase()) ||
        profile.specialties?.some(s => s.toLowerCase().includes(query.toLowerCase()))
      )
      
      return results
    } catch (err) {
      setError('Failed to search domains')
      return []
    } finally {
      setIsLoading(false)
    }
  }

  return {
    profile,
    isLoading,
    error,
    resolveDomain,
    checkAvailability,
    registerDomain,
    updateProfile,
    getDomainPrice,
    searchDomains,
  }
}