<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('system.title') }}</div>

    <div class="row q-col-gutter-md">
      <!-- 基础设置 -->
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('system.basicSettings') }}</div>
            <q-form @submit="saveBasicSettings" class="q-gutter-md">
              <q-input
                v-model="basicForm.siteName"
                :label="$t('system.siteName')"
                outlined
                :placeholder="$t('system.siteNamePlaceholder')"
              />
              <q-input
                v-model="basicForm.siteDescription"
                :label="$t('system.siteDescription')"
                outlined
                type="textarea"
                rows="3"
                :placeholder="$t('system.siteDescriptionPlaceholder')"
              />
              <div>
                <q-btn type="submit" color="primary" :label="$t('system.saveSettings')" :loading="savingBasic" />
              </div>
            </q-form>
          </q-card-section>
        </q-card>
      </div>

      <!-- 联系方式 -->
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('system.contactSettings') }}</div>
            <q-form @submit="saveContactSettings" class="q-gutter-md">
              <q-input
                v-model="contactForm.email"
                :label="$t('system.contactEmail')"
                outlined
                type="email"
                :placeholder="$t('system.contactEmailPlaceholder')"
              />
              <q-input
                v-model="contactForm.phone"
                :label="$t('system.contactPhone')"
                outlined
                type="tel"
                :placeholder="$t('system.contactPhonePlaceholder')"
              />
              <div>
                <q-btn type="submit" color="primary" :label="$t('system.saveSettings')" :loading="savingContact" />
              </div>
            </q-form>
          </q-card-section>
        </q-card>
      </div>

      <!-- 系统信息 -->
      <div class="col-12">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('system.systemInfo') }}</div>
            <q-list>
              <q-item>
                <q-item-section avatar>
                  <q-icon name="info" color="primary" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ $t('system.systemVersion') }}</q-item-label>
                  <q-item-label caption>v1.0.0</q-item-label>
                </q-item-section>
              </q-item>
              <q-item>
                <q-item-section avatar>
                  <q-icon name="code" color="primary" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ $t('system.frontendFramework') }}</q-item-label>
                  <q-item-label caption>Quasar + Vue 3 + TypeScript</q-item-label>
                </q-item-section>
              </q-item>
              <q-item>
                <q-item-section avatar>
                  <q-icon name="storage" color="primary" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ $t('system.backendFramework') }}</q-item-label>
                  <q-item-label caption>Drogon C++</q-item-label>
                </q-item-section>
              </q-item>
              <q-item>
                <q-item-section avatar>
                  <q-icon name="schedule" color="primary" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ $t('system.systemTime') }}</q-item-label>
                  <q-item-label caption>{{ currentTime }}</q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { getStorageItem, setStorageItem } from '@/utils/storage';

const $q = useQuasar();
const { t } = useI18n();

const savingBasic = ref(false);
const savingContact = ref(false);
const currentTime = ref('');

const basicForm = reactive({
  siteName: 'MyAI 管理后台',
  siteDescription: '基于 Quasar 框架构建的现代化管理系统',
});

const contactForm = reactive({
  email: 'admin@example.com',
  phone: '400-888-8888',
});

let timeInterval: ReturnType<typeof setInterval>;

/**
 * @brief 保存基础设置
 */
async function saveBasicSettings() {
  savingBasic.value = true;
  try {
    await new Promise((resolve) => setTimeout(resolve, 500));
    setStorageItem('admin_basic_settings', JSON.stringify(basicForm));
    $q.notify({ type: 'positive', message: t('system.settingsSaved') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  } finally {
    savingBasic.value = false;
  }
}

/**
 * @brief 保存联系方式
 */
async function saveContactSettings() {
  savingContact.value = true;
  try {
    await new Promise((resolve) => setTimeout(resolve, 500));
    setStorageItem('admin_contact_settings', JSON.stringify(contactForm));
    $q.notify({ type: 'positive', message: t('system.settingsSaved') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  } finally {
    savingContact.value = false;
  }
}

/**
 * @brief 更新系统时间
 */
function updateTime() {
  currentTime.value = new Date().toLocaleString();
}

/**
 * @brief 加载保存的设置
 */
function loadSettings() {
  const basic = getStorageItem<string>('admin_basic_settings', '');
  if (basic) {
    Object.assign(basicForm, JSON.parse(basic));
  }
  const contact = getStorageItem<string>('admin_contact_settings', '');
  if (contact) {
    Object.assign(contactForm, JSON.parse(contact));
  }
}

onMounted(() => {
  loadSettings();
  updateTime();
  timeInterval = setInterval(updateTime, 1000);
});

onUnmounted(() => {
  if (timeInterval) {
    clearInterval(timeInterval);
  }
});
</script>
