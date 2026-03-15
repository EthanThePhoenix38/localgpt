import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import Head from '@docusaurus/Head';
import { templates } from '@site/src/data/templates';
import styles from './styles.module.css';

// Type from the data file
interface Template {
  id: string;
  title: string;
  slug: string;
  category: 'fantasy' | 'sci-fi' | 'horror' | 'urban';
  description: string;
  longDescription: string;
  features: string[];
  customization: string[];
  faq: { question: string; answer: string }[];
  related: string[];
}

interface Props {
  template: Template;
}

export default function TemplatePage({ template }: Props) {
  // Find related templates by ID
  const relatedTemplates = template.related
    .map(id => templates.find(t => t.id === id))
    .filter((t): t is Template => Boolean(t));

  // Structured Data (JSON-LD)
  const schema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "Product",
        "name": template.title,
        "description": template.description,
        "offers": {
          "@type": "Offer",
          "price": "0",
          "priceCurrency": "USD",
          "availability": "https://schema.org/InStock"
        }
      },
      {
        "@type": "3DModel",
        "name": template.title,
        "encodingFormat": "model/gltf-binary",
        "contentUrl": `https://localgpt.app/assets/templates/${template.id}.glb`, 
        "description": template.description
      },
      {
        "@type": "BreadcrumbList",
        "itemListElement": [
          {
            "@type": "ListItem",
            "position": 1,
            "name": "Templates",
            "item": "https://localgpt.app/templates"
          },
          {
            "@type": "ListItem",
            "position": 2,
            "name": template.category.charAt(0).toUpperCase() + template.category.slice(1),
            "item": `https://localgpt.app/templates/${template.category}`
          },
          {
            "@type": "ListItem",
            "position": 3,
            "name": template.title
          }
        ]
      },
      {
        "@type": "FAQPage",
        "mainEntity": template.faq.map(q => ({
          "@type": "Question",
          "name": q.question,
          "acceptedAnswer": {
            "@type": "Answer",
            "text": q.answer
          }
        }))
      },
      {
        "@type": "HowTo",
        "name": `How to Customize ${template.title}`,
        "step": template.customization.map((step, i) => ({
          "@type": "HowToStep",
          "position": i + 1,
          "text": step
        }))
      }
    ]
  };

  return (
    <Layout
      title={template.title}
      description={template.description}>
      <Head>
        <script type="application/ld+json">
          {JSON.stringify(schema)}
        </script>
      </Head>
      
      <div className={styles.hero}>
        <div className="container">
          <h1 className={styles.heroTitle}>{template.title}</h1>
          <p className={styles.heroSubtitle}>{template.description}</p>
          
          <div className={styles.preview}>
            {/* Placeholder for future 3D viewer or image */}
            <div className={styles.previewPlaceholder}>
              Interactive 3D Preview
            </div>
          </div>

          <div className={styles.ctaWrapper}>
            <Link
              className="button button--primary button--lg"
              to="/docs/gen">
              Open in Editor
            </Link>
            <Link
              className="button button--secondary button--lg"
              to="/docs/intro">
              Documentation
            </Link>
          </div>
        </div>
      </div>

      <div className="container">
        <div className={styles.content}>
          <div className={styles.description}>
            {template.longDescription.trim().split('\n').map((line, i) => (
              line.trim() && <p key={i}>{line.trim()}</p>
            ))}
          </div>

          <div className={styles.featuresSection}>
            <h2>Features</h2>
            <div className={styles.featuresGrid}>
              {template.features.map((feature, i) => (
                <div key={i} className={styles.featureCard}>
                  <h3>{feature}</h3>
                </div>
              ))}
            </div>
          </div>

          <div className={styles.featuresSection}>
            <h2>How to Customize This World</h2>
            <div className={styles.featureCard} style={{ marginTop: '1.5rem' }}>
                <ol style={{ paddingLeft: '1.5rem', margin: 0 }}>
                    {template.customization.map((step, i) => (
                        <li key={i} style={{ marginBottom: '0.5rem', fontSize: '1.1rem' }}>
                            <span dangerouslySetInnerHTML={{ 
                                __html: step.replace(/`([^`]+)`/g, '<code>$1</code>') 
                            }} />
                        </li>
                    ))}
                </ol>
            </div>
          </div>

          <div className={styles.faqSection}>
            <h2>Frequently Asked Questions</h2>
            {template.faq.map((q, i) => (
              <div key={i} className={styles.faqItem}>
                <h3>{q.question}</h3>
                <p>{q.answer}</p>
              </div>
            ))}
          </div>

          <div className={styles.relatedSection}>
            <h2>Related Templates</h2>
            <div className={styles.relatedGrid}>
              {relatedTemplates.map(t => (
                <Link 
                  key={t.id} 
                  to={`/templates/${t.slug}`} 
                  className={styles.relatedCard}>
                  <h4>{t.title}</h4>
                  <p>{t.description}</p>
                </Link>
              ))}
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
