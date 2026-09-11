import { ref } from 'vue'

interface UseStepFormOptions {
  totalSteps: number
  onNext: (event: 'next') => void
  onPrev: (event: 'prev') => void
  onSubmit: (event: 'submit') => void
}

export function useStepForm(options: UseStepFormOptions) {
  const currentStep = ref(1)

  function nextStep(): void {
    if (currentStep.value < options.totalSteps) {
      currentStep.value++
      options.onNext('next')
    }
  }

  function prevStep(): void {
    if (currentStep.value > 1) {
      currentStep.value--
      options.onPrev('prev')
    }
  }

  function submit(): void {
    options.onSubmit('submit')
  }

  return {
    currentStep,
    nextStep,
    prevStep,
    submit
  }
}
