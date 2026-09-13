<template>
  <q-page class="login-page flex flex-center bg-grey-2">
    <q-card class="login-card" flat bordered>
      <q-card-section class="text-center q-pt-xl">
        <q-icon name="forum" size="64px" color="primary" />
        <div class="text-h5 q-mt-md text-weight-bold">Social</div>
        <div class="text-caption text-grey-6 q-mb-lg">登录到 Matrix 聊天网络</div>
      </q-card-section>

      <q-card-section class="q-px-xl q-pb-xl">
        <!-- Tab 切换 -->
        <q-tabs
          v-model="tab"
          dense
          class="text-grey"
          active-color="primary"
          indicator-color="primary"
          align="justify"
          narrow-indicator
        >
          <q-tab name="login" label="登录" />
          <q-tab name="register" label="注册" />
        </q-tabs>

        <q-separator class="q-mb-md" />

        <q-tab-panels v-model="tab" animated>
          <!-- 登录面板 -->
          <q-tab-panel name="login" class="q-px-none">
            <q-form @submit.prevent="handleLogin" class="q-gutter-md">
              <q-input
                v-model="form.username"
                label="用户名 / Matrix ID"
                placeholder="@username:server.tld"
                outlined
                dense
                :rules="[val => !!val || '请输入用户名']"
                :disable="auth.isLoading"
                autocomplete="username"
              >
                <template v-slot:prepend>
                  <q-icon name="person" />
                </template>
              </q-input>

              <q-input
                v-model="form.password"
                label="密码"
                :type="showPassword ? 'text' : 'password'"
                outlined
                dense
                :rules="[val => !!val || '请输入密码']"
                :disable="auth.isLoading"
                autocomplete="current-password"
              >
                <template v-slot:prepend>
                  <q-icon name="lock" />
                </template>
                <template v-slot:append>
                  <q-icon
                    :name="showPassword ? 'visibility_off' : 'visibility'"
                    class="cursor-pointer"
                    @click="showPassword = !showPassword"
                  />
                </template>
              </q-input>

              <div v-if="auth.error" class="text-negative text-caption">
                <q-icon name="error" class="q-mr-xs" />
                {{ auth.error }}
              </div>

              <q-btn
                type="submit"
                color="primary"
                class="full-width"
                :loading="auth.isLoading"
                label="登录"
                size="md"
              />

              <!-- SSO 登录 -->
              <q-separator class="q-my-sm" />
              <q-btn
                v-if="loginFlows.sso"
                type="button"
                color="accent"
                class="full-width"
                outline
                icon="person"
                label="使用 SSO 登录"
                size="md"
                :loading="auth.isLoading"
                @click="handleSsoLogin"
              />
            </q-form>
          </q-tab-panel>

          <!-- 注册面板 -->
          <q-tab-panel name="register" class="q-px-none">
            <q-form @submit.prevent="handleRegister" class="q-gutter-md">
              <q-input
                v-model="form.username"
                label="用户名"
                placeholder="仅字母、数字、下划线、连字符"
                outlined
                dense
                :rules="[
                  val => !!val || '请输入用户名',
                  val => /^[a-zA-Z0-9._=\-/+]+$/.test(val) || '用户名包含非法字符',
                ]"
                :disable="auth.isLoading"
                autocomplete="username"
              >
                <template v-slot:prepend>
                  <q-icon name="person" />
                </template>
              </q-input>

              <q-input
                v-model="form.displayName"
                label="显示名称（可选）"
                outlined
                dense
                :disable="auth.isLoading"
              >
                <template v-slot:prepend>
                  <q-icon name="badge" />
                </template>
              </q-input>

              <q-input
                v-model="form.password"
                label="密码"
                :type="showPassword ? 'text' : 'password'"
                outlined
                dense
                :rules="[
                  val => !!val || '请输入密码',
                  val => val.length >= 8 || '密码至少 8 个字符',
                ]"
                :disable="auth.isLoading"
                autocomplete="new-password"
              >
                <template v-slot:prepend>
                  <q-icon name="lock" />
                </template>
                <template v-slot:append>
                  <q-icon
                    :name="showPassword ? 'visibility_off' : 'visibility'"
                    class="cursor-pointer"
                    @click="showPassword = !showPassword"
                  />
                </template>
              </q-input>

              <q-input
                v-model="form.confirmPassword"
                label="确认密码"
                :type="showPassword ? 'text' : 'password'"
                outlined
                dense
                :rules="[
                  val => !!val || '请确认密码',
                  val => val === form.password || '两次密码不一致',
                ]"
                :disable="auth.isLoading"
                autocomplete="new-password"
              >
                <template v-slot:prepend>
                  <q-icon name="lock" />
                </template>
              </q-input>

              <div v-if="auth.error" class="text-negative text-caption">
                <q-icon name="error" class="q-mr-xs" />
                {{ auth.error }}
              </div>

              <q-btn
                type="submit"
                color="primary"
                class="full-width"
                :loading="auth.isLoading"
                label="注册"
                size="md"
              />
            </q-form>
          </q-tab-panel>
        </q-tab-panels>

        <!-- 底部服务器信息 -->
        <div class="text-center q-mt-lg text-caption text-grey-5">
          连接到:
          <a
            :href="serverUrl"
            target="_blank"
            class="text-primary text-caption"
          >{{ serverUrl }}</a>
        </div>
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { useAuthLogin } from '@erp-new-frontend-monorepo/composables/src/useAuthLogin';

const {
  tab,
  showPassword,
  serverUrl,
  loginFlows,
  form,
  auth,
  handleLogin,
  handleRegister,
  handleSsoLogin,
} = useAuthLogin();
</script>

<style scoped>
.login-page {
  min-height: 100vh;
}

.login-card {
  width: 100%;
  max-width: 420px;
  border-radius: 12px;
}
</style>
