import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useQuasar } from 'quasar'
import { useI18n } from 'vue-i18n'

interface LoginCredentials {
  username: string
  password: string
  remember: boolean
}

export function useLogin() {
  const router = useRouter()
  const $q = useQuasar()
  const { t: $t } = useI18n()

  const loading = ref(false)

  async function handleLogin(credentials: LoginCredentials): Promise<void> {
    loading.value = true

    try {
      // 模拟登录请求
      await new Promise(resolve => setTimeout(resolve, 1000))

      $q.notify({
        type: 'positive',
        message: $t('login.loginSuccess')
      })

      router.push('/dashboard')
    } catch (error) {
      $q.notify({
        type: 'negative',
        message: $t('login.loginFailed')
      })
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    handleLogin
  }
}
