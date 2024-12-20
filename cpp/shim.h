#include <memory>
#include <xapian.h>

#ifndef _XAPIAN_SHIM_H
#define _XAPIAN_SHIM_H

namespace shim {
  enum WildcardLimitBehavior {
    WILDCARD_LIMIT_ERROR = Xapian::Query::WILDCARD_LIMIT_ERROR,
    WILDCARD_LIMIT_FIRST = Xapian::Query::WILDCARD_LIMIT_FIRST,
    WILDCARD_LIMIT_MOST_FREQUENT = Xapian::Query::WILDCARD_LIMIT_MOST_FREQUENT,
  };

  class FfiExpandDecider : public Xapian::ExpandDecider {
    public:
      FfiExpandDecider() : Xapian::ExpandDecider() {}
      virtual bool operator()(const std::string &term) const override { return this->should_keep(term); }
      virtual bool should_keep(const std::string&) const = 0;
  };

  class FfiFieldProcessor : public Xapian::FieldProcessor {
    public:
      FfiFieldProcessor() : Xapian::FieldProcessor() {}
      virtual FfiFieldProcessor* upcast() { return this; }
      virtual Xapian::Query operator()(const std::string &str) { return this->process(str); }
      virtual Xapian::Query process(const std::string &str) const = 0;
  };

  class FfiMatchDecider : public Xapian::MatchDecider {
    public:
      FfiMatchDecider() : Xapian::MatchDecider() {}
      virtual bool operator()(const Xapian::Document &doc) const override { return this->is_match(doc); }
      virtual bool is_match(const Xapian::Document&) const = 0;
  };

  class FfiMatchSpy : public Xapian::MatchSpy {
    public:
      FfiMatchSpy() : Xapian::MatchSpy() {}
      virtual FfiMatchSpy* upcast() { return this; }
      virtual std::string name() const override { return std::string("shim::FfiMatchSpy"); }
      virtual void operator()(const Xapian::Document &doc, double wt) override { return this->observe(doc, wt); }
      virtual void observe(const Xapian::Document&, double) = 0;
  };

  class FfiRangeProcessor: public Xapian::RangeProcessor {
    public:
      FfiRangeProcessor(Xapian::valueno slot, std::string &marker, unsigned flags) : Xapian::RangeProcessor(slot, marker, flags) {}
      virtual FfiRangeProcessor* upcast() { return this; }
      virtual Xapian::Query operator()(const std::string &begin, const std::string &end) override { return *(this->process_range(begin, end)); }
      virtual std::unique_ptr<Xapian::Query> process_range(const std::string&, const std::string&) = 0;
  };

  class FfiStopper : public Xapian::Stopper {
    public:
      FfiStopper() : Xapian::Stopper() {}
      virtual bool operator()(const std::string &word) const override { return this->is_stopword(word); }
      virtual bool is_stopword(const std::string&) const = 0;
  };

  inline Xapian::Database database_clone(const Xapian::Database &db) { return Xapian::Database(db); }

  inline Xapian::RangeProcessor& date_range_processor_upcast(Xapian::DateRangeProcessor &rp) { return rp; }

  inline Xapian::Document document_copy(const Xapian::Document &doc) { return Xapian::Document(doc); }

  inline void enquire_add_matchspy( Xapian::Enquire &e, FfiMatchSpy *m) { e.add_matchspy(m); }
  inline Xapian::ESet enquire_get_eset(
      const Xapian::Enquire &e, Xapian::termcount maxitems, const Xapian::RSet &rset,
      int flags, const FfiExpandDecider *decider, double min_wt
  ) { return e.get_eset(maxitems, rset, flags, decider, min_wt); }
  inline Xapian::MSet enquire_get_mset(
      const Xapian::Enquire &e, Xapian::doccount first,
      Xapian::doccount maxitems,  Xapian::doccount atleast,
      const Xapian::RSet *rset, const FfiMatchDecider *decider
  ) { return e.get_mset(first, maxitems, atleast, rset, decider); }

  inline Xapian::ESetIterator eset_iterator_copy(const Xapian::ESetIterator &it) { return Xapian::ESetIterator(it); }
  inline void eset_iterator_decrement(Xapian::ESetIterator &it) { it--; }
  inline bool eset_iterator_eq(const Xapian::ESetIterator &a, const Xapian::ESetIterator &b) { return a == b; }
  inline void eset_iterator_increment(Xapian::ESetIterator &it) { it++; }
  inline std::string eset_iterator_term(const Xapian::ESetIterator &it) { return *it; }

  inline Xapian::MSetIterator mset_iterator_copy(const Xapian::MSetIterator &it) { return Xapian::MSetIterator(it); }
  inline void mset_iterator_decrement(Xapian::MSetIterator &it) { it--; }
  inline Xapian::docid mset_iterator_docid(const Xapian::MSetIterator &it) { return *it; }
  inline bool mset_iterator_eq(const Xapian::MSetIterator &a, const Xapian::MSetIterator &b) { return a == b; }
  inline void mset_iterator_increment(Xapian::MSetIterator &it) { it++; }

  inline Xapian::RangeProcessor& number_range_processor_upcast(Xapian::NumberRangeProcessor &rp) { return rp; }

  inline Xapian::PositionIterator position_iterator_copy(const Xapian::PositionIterator &it) { return Xapian::PositionIterator(it); }
  inline bool position_iterator_eq(const Xapian::PositionIterator &a, const Xapian::PositionIterator &b) { return a == b; }
  inline void position_iterator_increment(Xapian::PositionIterator &it) { it++; }
  inline Xapian::termpos position_iterator_position(const Xapian::PositionIterator &it) { return *it; }

  inline Xapian::Query query_clone(const Xapian::Query &q) { return Xapian::Query(q); }

  inline void query_parser_set_stopper(Xapian::QueryParser &qp, const FfiStopper *stopper) { qp.set_stopper(stopper); }
  inline void query_parser_add_boolean_prefix(
      Xapian::QueryParser &qp, const std::string &field,
      FfiFieldProcessor *proc, const std::string *grouping
  ) { return qp.add_boolean_prefix(field, proc, grouping); }
  inline void query_parser_add_prefix(Xapian::QueryParser &qp, const std::string &field, FfiFieldProcessor *proc) {
    return qp.add_prefix(field, proc);
  }
  inline void query_parser_add_range_processor(Xapian::QueryParser &qp, FfiRangeProcessor *rp, std::string *grouping) {
    return qp.add_rangeprocessor(rp, grouping);
  }

  inline Xapian::Query range_processor_evaluate_range(Xapian::RangeProcessor &rp, const std::string &start, const std::string &end) { return rp(start, end); }

  inline std::string stemmer_stem(const Xapian::Stem &stem, const std::string &word) { return stem(word); }

  inline void term_generator_set_stopper(Xapian::TermGenerator &tg, const FfiStopper *stopper) { return tg.set_stopper(stopper); }

  inline Xapian::TermIterator term_iterator_copy(const Xapian::TermIterator &it) { return Xapian::TermIterator(it); }
  inline bool term_iterator_eq(const Xapian::TermIterator &a, const Xapian::TermIterator &b) { return a == b; }
  inline void term_iterator_increment(Xapian::TermIterator &it) { it++; }
  inline std::string term_iterator_term(const Xapian::TermIterator &it) { return *it; }

  inline int wildcard_limit_behavior_to_int(const WildcardLimitBehavior b) { return b; }

  inline const Xapian::Database& writable_database_upcast(const Xapian::WritableDatabase &db) { return db; }
}

#endif
