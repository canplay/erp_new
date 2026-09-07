{{/*
=============================================================================
MyAI Backend Helm Chart 辅助函数模板
=============================================================================
*/}}

{{/*
根据服务名称获取副本数
*/}}
{{- define "myai.replicaCount" -}}
{{- $replicaCount := .Values.replicaCount -}}
{{- $serviceName := .serviceName -}}
{{- if hasKey $replicaCount $serviceName -}}
{{- index $replicaCount $serviceName -}}
{{- else -}}
1
{{- end -}}
{{- end -}}

{{/*
根据服务名称获取资源限制
*/}}
{{- define "myai.resources" -}}
{{- $resources := .Values.resources -}}
{{- $serviceName := .serviceName -}}
{{- if hasKey $resources $serviceName -}}
{{- index $resources $serviceName | toYaml }}
{{- else if hasKey $resources "default" -}}
{{- index $resources "default" | toYaml }}
{{- end -}}
{{- end -}}

{{/*
获取镜像完整地址
支持按服务独立配置 imageName，无则使用全局 image.repository
*/}}
{{- define "myai.image" -}}
{{- $registry := .Values.global.imageRegistry -}}
{{- $serviceConfig := .serviceConfig -}}
{{- $imageName := $serviceConfig.imageName | default .Values.image.repository -}}
{{- $tag := $serviceConfig.tag | default .Values.image.tag -}}
{{- /* 修复 (2026-08-07): 纯数字 tag (如 202608070337) 经 --set 传入会被 Helm
       解析为 int64, printf %s 输出 %!s(int64=...); toString 强制字符串 */ -}}
{{- $tagStr := toString $tag -}}
{{- if $registry -}}
{{- printf "%s/%s:%s" $registry $imageName $tagStr -}}
{{- else -}}
{{- printf "%s:%s" $imageName $tagStr -}}
{{- end -}}
{{- end -}}

{{/*
获取服务名称
*/}}
{{- define "myai.fullname" -}}
{{- $name := .componentName | default .serviceName -}}
{{- printf "%s-%s" (include "myai.name" .) $name -}}
{{- end -}}

{{/*
获取 Chart 名称
*/}}
{{- define "myai.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
获取 Chart 完整名称
*/}}
{{- define "myai.fullchartname" -}}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
获取服务标签
*/}}
{{- define "myai.labels" -}}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
{{ include "myai.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
获取选择器标签
*/}}
{{- define "myai.selectorLabels" -}}
app.kubernetes.io/name: {{ include "myai.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- if .serviceName }}
app.kubernetes.io/component: {{ .serviceName }}
{{- end }}
{{- end -}}

{{/*
获取环境变量
*/}}
{{- define "myai.envVars" -}}
{{- range $key, $value := .env }}
- name: {{ $key }}
  value: {{ $value | quote }}
{{- end }}
{{- end -}}

{{/*
从 ConfigMap 获取环境变量
*/}}
{{- define "myai.envFromConfigMap" -}}
{{- if .configMapName -}}
- configMapRef:
    name: {{ .configMapName }}
{{- end -}}
{{- end -}}

{{/*
从 Secret 获取环境变量
*/}}
{{- define "myai.envFromSecret" -}}
{{- if .secretName -}}
- secretRef:
    name: {{ .secretName }}
{{- end -}}
{{- end -}}

{{/*
获取健康检查配置
*/}}
{{- define "myai.livenessProbe" -}}
{{- if .Values.healthCheck.enabled -}}
livenessProbe:
  httpGet:
    path: /health
    port: {{ .httpPort }}
  initialDelaySeconds: {{ .Values.healthCheck.livenessProbe.initialDelaySeconds }}
  periodSeconds: {{ .Values.healthCheck.livenessProbe.periodSeconds }}
  timeoutSeconds: {{ .Values.healthCheck.livenessProbe.timeoutSeconds }}
  failureThreshold: {{ .Values.healthCheck.livenessProbe.failureThreshold }}
  successThreshold: {{ .Values.healthCheck.livenessProbe.successThreshold }}
{{- end -}}
{{- end -}}

{{- define "myai.readinessProbe" -}}
{{- if .Values.healthCheck.enabled -}}
readinessProbe:
  httpGet:
    path: /health
    port: {{ .httpPort }}
  initialDelaySeconds: {{ .Values.healthCheck.readinessProbe.initialDelaySeconds }}
  periodSeconds: {{ .Values.healthCheck.readinessProbe.periodSeconds }}
  timeoutSeconds: {{ .Values.healthCheck.readinessProbe.timeoutSeconds }}
  failureThreshold: {{ .Values.healthCheck.readinessProbe.failureThreshold }}
  successThreshold: {{ .Values.healthCheck.readinessProbe.successThreshold }}
{{- end -}}
{{- end -}}

{{/*
获取挂载卷
*/}}
{{- define "myai.volumeMounts" -}}
{{- if .volumeMounts -}}
{{- toYaml .volumeMounts | nindent 8 -}}
{{- end -}}
{{- end -}}

{{/*
获取卷
*/}}
{{- define "myai.volumes" -}}
{{- if .volumes -}}
{{- toYaml .volumes | nindent 8 -}}
{{- end -}}
{{- end -}}
