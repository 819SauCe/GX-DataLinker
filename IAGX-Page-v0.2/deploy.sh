#!/bin/bash

echo "🔄 Iniciando deploy do IAGX..."

cd /opt/IAGX-Page-v0.2 || { echo "❌ Diretório não encontrado"; exit 1; }

echo "🧹 Removendo build anterior..."
rm -rf build

echo "📦 Gerando nova build..."
npm run build || { echo "❌ Falha ao gerar build"; exit 1; }

echo "🔁 Recarregando Nginx..."
nginx -t && systemctl reload nginx || { echo "❌ Erro ao recarregar Nginx"; exit 1; }

echo "🚀 Reiniciando backend com PM2..."
pm2 restart backend-iagx || { echo "❌ Falha ao reiniciar o backend"; exit 1; }

echo "✅ Deploy concluído com sucesso!"
