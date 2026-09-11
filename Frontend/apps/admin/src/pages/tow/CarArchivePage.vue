<template>
  <q-page class="q-pa-md">
    <q-breadcrumbs class="q-mb-md">
      <q-breadcrumbs-el :label="$t('parking.towTasks')" to="/tow" />
      <q-breadcrumbs-el :label="$t('tow.vehicleNumberPrefix') + carId" />
    </q-breadcrumbs>

    <SkeletonLoader v-if="store.loading" type="card" :rows="6" />
    <template v-else-if="car">
      <div class="row q-gutter-md">
        <q-card flat bordered class="col-6">
          <q-card-section><div class="text-h6">{{ $t('tow.carInfo') }}</div></q-card-section>
          <q-separator />
          <q-card-section>
            <q-list dense>
              <q-item><q-item-section><q-item-label>{{ $t('tow.plateNo') }}</q-item-label><q-item-label caption class="text-h6 text-primary">{{ car.license }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.carTypeColor') }}</q-item-label><q-item-label caption>{{ car.car_type }} / {{ car.car_color }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.engineNumber') }}</q-item-label><q-item-label caption>{{ car.engine }}</q-item-label></q-item-section></q-item>
            </q-list>
          </q-card-section>
        </q-card>

        <q-card flat bordered class="col-6">
          <q-card-section><div class="text-h6">{{ $t('tow.towInfo') }}</div></q-card-section>
          <q-separator />
          <q-card-section>
            <q-list dense>
              <q-item><q-item-section><q-item-label>{{ $t('tow.towType') }}</q-item-label><q-item-label caption>{{ car.dc_type }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.towCause') }}</q-item-label><q-item-label caption>{{ car.dc_causes }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.towDate') }}</q-item-label><q-item-label caption>{{ car.dc_date }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.towAddress') }}</q-item-label><q-item-label caption>{{ car.dc_address }}</q-item-label></q-item-section></q-item>
            </q-list>
          </q-card-section>
        </q-card>

        <q-card flat bordered class="col-6">
          <q-card-section><div class="text-h6">{{ $t('tow.partyInfo') }}</div></q-card-section>
          <q-separator />
          <q-card-section>
            <q-list dense>
              <q-item><q-item-section><q-item-label>{{ $t('tow.partyName') }}</q-item-label><q-item-label caption>{{ car.dc_party_name }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.idNumber') }}</q-item-label><q-item-label caption>{{ car.dc_party_cardid }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.phone') }}</q-item-label><q-item-label caption>{{ car.dc_party_tel }}</q-item-label></q-item-section></q-item>
            </q-list>
          </q-card-section>
        </q-card>

        <q-card flat bordered class="col-6">
          <q-card-section><div class="text-h6">{{ $t('tow.operatorInfo') }}</div></q-card-section>
          <q-separator />
          <q-card-section>
            <q-list dense>
              <q-item><q-item-section><q-item-label>{{ $t('tow.operator') }}</q-item-label><q-item-label caption>{{ car.operator }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.driver') }}</q-item-label><q-item-label caption>{{ car.driver }}</q-item-label></q-item-section></q-item>
              <q-item><q-item-section><q-item-label>{{ $t('tow.remark') }}</q-item-label><q-item-label caption>{{ car.remark || '-' }}</q-item-label></q-item-section></q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>
    </template>
    <EmptyState v-else icon="local_shipping" :title="$t('empty.noData')" :description="$t('tow.carArchiveNotFound')" />
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useTowStore } from '@/stores/tow';
import SkeletonLoader from '@/components/SkeletonLoader.vue';
import EmptyState from '@/components/EmptyState.vue';

const route = useRoute();
const store = useTowStore();
const carId = computed(() => Number(route.params.id));
const car = computed(() => store.currentCar);

onMounted(() => store.fetchCar(carId.value));
</script>
