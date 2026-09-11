<template>
  <q-page class="flex flex-center">
    <div class="text-center">
      <div class="text-h1 text-weight-bold text-negative q-mb-md">403</div>
      <div class="text-h4 q-mb-lg text-grey-7">{{ $t('error.403.title') }}</div>
      <div class="text-body1 text-grey-6 q-mb-lg">
        {{ $t('error.403.message') }}
      </div>
      <q-btn color="primary" :label="$t('error.403.backHome')" icon="home" to="/" />
    </div>
  </q-page>
</template>
